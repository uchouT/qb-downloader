package moe.uchout.qbdownloader.util;

import org.junit.jupiter.api.Test;

import java.util.*;
import moe.uchout.qbdownloader.entity.*;
import moe.uchout.qbdownloader.util.uploader.UploaderFactory;
import java.io.File;

public class UtilTest {

        @Test
        public static void main(String[] args) {
                // ConfigUtil.sync();
                // String hash = "2896014fcadfb8d5e81689f9482b4226be664aea";
                // ConfigUtil.load();
                // System.out.println(ConfigUtil.CONFIG.getQbHost());
                // System.out.println(ConfigUtil.CONFIG.getQbUsername());
                // System.out.println(ConfigUtil.CONFIG.getQbPassword());
                // System.out.println(ConfigUtil.CONFIG.getAlistHost());
                // System.out.println(ConfigUtil.CONFIG.getAlistToken());
                // System.out.println(ConfigUtil.CONFIG.getRcloneHost());
                // System.out.println(ConfigUtil.CONFIG.getRclonePassword());
                // System.out.println(ConfigUtil.CONFIG.getRcloneuserName());
                ConfigUtil.CONFIG.setQbHost("http://localhost:8080").setQbUsername("admin").setQbPassword("secret")
                                .setRcloneHost("http://localhost:5572")
                                .setRcloneuserName("admin")
                                .setRclonePassword("secret");
                QbUtil.login();
                TaskUtil.addTask(
                                "magnet:?xt=urn:btih:68ebf1641e69a5b5c9eddbe75fc918ccda5ebf88&dn=%5B%E5%8C%97%E5%AE%87%E6%B2%BB%E5%AD%97%E5%B9%95%E7%BB%84%26LoliHouse%5D%20%E5%9C%B0%E3%80%82-%E5%85%B3%E4%BA%8E%E5%9C%B0%E7%90%83%E7%9A%84%E8%BF%90%E5%8A%A8-%20%2F%20Chi.%20Chikyuu%20no%20Undou%20ni%20Tsuite%20%5B01-25%20%E4%BF%AE%E6%AD%A3%E5%90%88%E9%9B%86%5D%5BWebRip%201080p%20HEVC-10bit%20AAC%20ASSx2%5D%5B%E7%AE%80%E7%B9%81%E6%97%A5%E5%86%85%E5%B0%81%E5%AD%97%E5%B9%95%5D%5BFin%5D&tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce",
                                "rclone", "E:\\Download", "od:/Test", 1000);
                Thread thread = new TaskUtil();
                thread.start();
        }
}