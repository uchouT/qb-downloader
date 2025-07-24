package moe.uchout.qbdownloader.util;

import java.io.FileInputStream;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import moe.uchout.qbdownloader.exception.SingleFileException;
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
        Map<String, Object> info = infoObj.value;
        @SuppressWarnings("unchecked")
        List<LinkedHashMap<String, Object>> files = (ArrayList<LinkedHashMap<String, Object>>) info
                .get("files");
        if (ObjectUtil.isNull(files)) {
            throw new SingleFileException("种子为单文件种子");
        }
        List<Long> fileLengthList = files.stream().map(file -> {
            return (Long) file.get("length");
        }).toList();
        return fileLengthList;
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
}

class TorrentInfoObj {
    TorrentInfoObj(Map<String, Object> value) {
        this.value = value;
    };

    Map<String, Object> value;
}
