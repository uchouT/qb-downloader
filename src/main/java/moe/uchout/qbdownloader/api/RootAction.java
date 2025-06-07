package moe.uchout.qbdownloader.api;

import moe.uchout.qbdownloader.annotation.Auth;
import cn.hutool.core.collection.CollUtil;
import cn.hutool.core.collection.EnumerationIter;
import cn.hutool.core.io.FileUtil;
import cn.hutool.core.io.resource.ResourceUtil;
import cn.hutool.core.util.StrUtil;
import cn.hutool.core.util.URLUtil;
import cn.hutool.http.ContentType;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import cn.hutool.system.OsInfo;
import cn.hutool.system.SystemUtil;
import lombok.Cleanup;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Objects;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;

/**
 * 网页处理
 */
@Auth(value = false)
@Slf4j
public class RootAction implements BaseAction {

    private static final String DEFAULT_INDEX_FILE_NAME = "index.html";

    private final String rootDir;

    private final List<String> indexFileNames;

    public RootAction() {
        this("dist", DEFAULT_INDEX_FILE_NAME);
    }

    public RootAction(String rootDir) {
        this(rootDir, DEFAULT_INDEX_FILE_NAME);
    }

    public RootAction(String rootDir, String... indexFileNames) {
        this.rootDir = rootDir;
        this.indexFileNames = CollUtil.toList(indexFileNames);
    }

    @Override
    public void doAction(HttpServerRequest request, HttpServerResponse response) {
        String path = request.getPath();
        String fileName = rootDir + path;

        Boolean ok = file(response, fileName, true);
        if (!ok) {
            response.send404("404 Not Found !");
        }
    }

    public Boolean file(HttpServerResponse response, String fileName, Boolean index) {
        log.debug(fileName);
        try {
            EnumerationIter<URL> resourceIter = ResourceUtil.getResourceIter(fileName);
            for (URL url : resourceIter) {
                @Cleanup
                InputStream inputStream = toInputStream(url, fileName);
                if (Objects.isNull(inputStream)) {
                    continue;
                }
                String mimeType = FileUtil.getMimeType(fileName);
                mimeType = StrUtil.blankToDefault(mimeType, ContentType.OCTET_STREAM.getValue());
                response.write(inputStream, mimeType);
                return true;
            }
            if (!index) {
                return false;
            }
            for (String indexFileName : indexFileNames) {
                Boolean ok = file(response, fileName + indexFileName, false);
                if (ok) {
                    return true;
                }
            }
        } catch (Exception e) {
            log.error(e.getMessage(), e);
        }
        return false;
    }

    public InputStream toInputStream(URL url, String fileName) throws IOException {
        String protocol = url.getProtocol();

        InputStream inputStream = null;
        if (protocol.equals("file")) {
            File file = new File(URLUtil.decode(url.getFile(), StandardCharsets.UTF_8));
            if (!file.isDirectory()) {
                inputStream = FileUtil.getInputStream(file);
            }
        } else {
            try {
                File jarFile = getJar();
                if (jarFile == null || !jarFile.exists()) {
                    log.warn("JAR file not found: {}", jarFile);
                    return null;
                }

                JarFile jar = new JarFile(jarFile);
                JarEntry jarEntry = jar.getJarEntry(fileName);
                if (Objects.isNull(jarEntry)) {
                    jar.close();
                    return null;
                }
                if (!jarEntry.isDirectory()) {
                    inputStream = new JarFileInputStream(jar, jar.getInputStream(jarEntry));
                } else {
                    jar.close();
                }
            } catch (Exception e) {
                log.error("Failed to read from JAR file: {}", e.getMessage(), e);
                return null;
            }
        }
        return inputStream;
    }

    private static class JarFileInputStream extends InputStream {
        private final JarFile jarFile;
        private final InputStream inputStream;

        public JarFileInputStream(JarFile jarFile, InputStream inputStream) {
            this.jarFile = jarFile;
            this.inputStream = inputStream;
        }

        @Override
        public int read() throws IOException {
            return inputStream.read();
        }

        @Override
        public int read(byte[] b) throws IOException {
            return inputStream.read(b);
        }

        @Override
        public int read(byte[] b, int off, int len) throws IOException {
            return inputStream.read(b, off, len);
        }

        @Override
        public void close() throws IOException {
            try {
                inputStream.close();
            } finally {
                jarFile.close();
            }
        }
    }

    public static File getJar() {
        try {
            String classPath = System.getProperty("java.class.path");
            if (classPath == null || classPath.isEmpty()) {
                return null;
            }

            OsInfo osInfo = SystemUtil.getOsInfo();
            String separator = osInfo.isWindows() ? ";" : ":";
            String jarPath = classPath.split(separator)[0];

            File jarFile = new File(jarPath);
            return jarFile.exists() ? jarFile : null;
        } catch (Exception e) {
            log.error("Failed to get JAR file: {}", e.getMessage(), e);
            return null;
        }
    }

}
