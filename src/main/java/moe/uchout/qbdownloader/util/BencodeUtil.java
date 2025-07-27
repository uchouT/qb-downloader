package moe.uchout.qbdownloader.util;

import java.io.FileInputStream;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import moe.uchout.qbdownloader.exception.SingleFileException;
import moe.uchout.qbdownloader.entity.TorrentContentNode;
import com.dampcake.bencode.Bencode;
import com.dampcake.bencode.Type;

import cn.hutool.core.util.ObjectUtil;

public class BencodeUtil {

    /**
     * 返回 fileName 路径下的 .torrent 文件的 infoObj
     * 
     * @param fileName
     * @return
     */
    public static TorrentInfoObj getInfo(String fileName) {
        try (FileInputStream fis = new FileInputStream(fileName)) {
            Bencode bencode = new Bencode();
            byte[] data = fis.readAllBytes();
            Map<String, Object> torrentData = bencode.decode(data, Type.DICTIONARY);
            Object infoObj = torrentData.get("info");
            @SuppressWarnings("unchecked")
            Map<String, Object> info = (infoObj instanceof Map) ? (Map<String, Object>) infoObj : new HashMap<>();
            return new TorrentInfoObj(info);
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }

    /**
     * 根据 infoObj 获取长度列表
     * 
     * @param infoObj
     * @return
     * @throws SingleFileException
     */
    public static List<Long> getFileLengthList(TorrentInfoObj infoObj) throws SingleFileException {

        List<Map<String, Object>> files = getFiles(infoObj);
        List<Long> fileLengthList = files.stream().map(file -> {
            return (Long) file.get("length");
        }).toList();
        return fileLengthList;
    }

    private static List<Map<String, Object>> getFiles(TorrentInfoObj infoObj) throws SingleFileException {
        Map<String, Object> info = infoObj.value;
        @SuppressWarnings("unchecked")
        List<Map<String, Object>> files = (List<Map<String, Object>>) info
                .get("files");
        if (ObjectUtil.isNull(files)) {
            throw new SingleFileException("种子为单文件");
        }
        return files;
    }

    /**
     * 返回种子的根目录
     * 
     * @param infoObj
     * @return
     */
    public static String getRootDir(TorrentInfoObj infoObj) {
        Map<String, Object> info = infoObj.value;
        return (String) info.get("name");
    }

    private static List<List<String>> getPathList(TorrentInfoObj infoObj) throws SingleFileException {
        List<Map<String, Object>> files = getFiles(infoObj);
        @SuppressWarnings("unchecked")
        List<List<String>> pathList = files.stream().map(m -> (List<String>) m.get("path"))
                .collect(Collectors.toList());
        return pathList;
    }

    /**
     * 获取种子文件内容树
     * 
     * @param filename .torrent 文件
     * @param rootDir  .torrent 文件的 rootDir，不可以是单文件的种子
     * @return
     * @throws SingleFileException
     */
    public static List<TorrentContentNode> getContentTree(String filename) throws SingleFileException {
        TorrentInfoObj infoObj = getInfo(filename);
        String rootDir = getRootDir(infoObj);
        TorrentContentNode Tree = buildMapedTree(infoObj, rootDir);
        ContentNodeMapToList(Tree);
        sortChildren(Tree);
        return List.of(Tree);
    }

    private static void ContentNodeMapToList(TorrentContentNode node) {
        if (ObjectUtil.isNotEmpty(node.childrenMap)) {
            node.children = new ArrayList<>(node.childrenMap.values());
            for (TorrentContentNode childNode : node.children) {
                ContentNodeMapToList(childNode);
            }
            node.childrenMap = null;
        }
    }

    private static TorrentContentNode buildMapedTree(TorrentInfoObj infoObj, String rootDir)
            throws SingleFileException {
        List<List<String>> pathList = getPathList(infoObj);
        TorrentContentNode root = new TorrentContentNode(-1, rootDir);
        int _folder = -2;
        for (int i = 0, size = pathList.size(); i < size; ++i) {
            TorrentContentNode currentNode = root;
            List<String> currentFile = pathList.get(i);
            for (int j = 0, length = currentFile.size(); j < length; ++j) {
                int id;
                String label = currentFile.get(j);
                if (j < length - 1) { // 非最后一个节点
                    id = _folder--;
                } else {
                    id = i;
                }
                currentNode.childrenMap.putIfAbsent(label, new TorrentContentNode(id, label));
                currentNode = currentNode.childrenMap.get(label);
            }
        }
        return root;
    }

    private static void sortChildren(TorrentContentNode node) {
        if (ObjectUtil.isEmpty(node.children)) {
            return;
        }
        node.children.sort((a, b) -> {
            boolean aIsFolder = ObjectUtil.isNotEmpty(a.children);
            boolean bIsFolder = ObjectUtil.isNotEmpty(b.children);
            if (aIsFolder != bIsFolder) {
                return bIsFolder ? 1 : -1;
            }
            return (a.label).compareTo(b.label);
        });

        for (TorrentContentNode child : node.children) {
            if (ObjectUtil.isEmpty(child.children)) {
                break;
            }
            sortChildren(child);
        }
    }
}

class TorrentInfoObj {
    TorrentInfoObj(Map<String, Object> value) {
        this.value = value;
    };

    Map<String, Object> value;
}
