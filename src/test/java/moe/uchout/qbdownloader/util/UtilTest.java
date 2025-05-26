package moe.uchout.qbdownloader.util;

import org.junit.jupiter.api.Test;

import java.util.*;
import moe.uchout.qbdownloader.entity.*;
import moe.uchout.qbdownloader.util.uploader.Alist;
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
                ConfigUtil.CONFIG.setQbHost("http://localhost:8080").setQbUsername("admin").setQbPassword("adminadmin")
                                .setAlistHost("http://localhost:5244")
                                .setRcloneHost("http://localhost:5572")
                                .setRcloneuserName("admin")
                                .setRclonePassword("secret");
                try {
                        QbUtil.login();
                } catch (Exception e) {
                        e.printStackTrace();
                }
                // TaskUtil.addTask(
                //                 "magnet:?xt=urn:btih:e4617eb881fc9f6c1f575a002e812c9d71427a72&dn=%5B%E6%BE%84%E7%A9%BA%E5%AD%A6%E5%9B%AD%26%E5%8A%A8%E6%BC%AB%E5%9B%BD%E5%AD%97%E5%B9%95%E7%BB%84%26LoliHouse%5D%20%E7%81%B0%E8%89%B2%3A%20%E5%B9%BB%E5%BD%B1%E6%89%B3%E6%9C%BA%20%2F%20%E3%82%B0%E3%83%AA%E3%82%B6%E3%82%A4%E3%82%A2%3A%20%E3%83%95%E3%82%A1%E3%83%B3%E3%83%88%E3%83%A0%E3%83%88%E3%83%AA%E3%82%AC%E3%83%BC%20%2F%20Grisaia%3A%20Phantom%20Trigger%20%5B01-13%20%E5%90%88%E9%9B%86%5D%5BWebRip%201080p%20HEVC-10bit%20AAC%5D%5B%E7%AE%80%E7%B9%81%E5%86%85%E5%B0%81%E5%AD%97%E5%B9%95%5D&tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce",
                //                 "rclone", "/home/i/Downloads", "od:/Test", 1000);
                // Thread thread = new TaskThread();
                // thread.start();
        }
}