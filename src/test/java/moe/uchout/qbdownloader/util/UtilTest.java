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
                                "magnet:?xt=urn:btih:601939063a0e74b12d0c6024cff4560d10ca76c6&dn=%E3%80%90%E5%96%B5%E8%90%8C%E5%A5%B6%E8%8C%B6%E5%B1%8B%E3%80%91%E2%98%8504%E6%9C%88%E6%96%B0%E7%95%AA%E2%98%85%5B%E6%90%9E%E7%AC%91%E6%BC%AB%E7%94%BB%E6%97%A5%E5%92%8CGO%20%2F%20%E6%90%9E%E7%AC%91%E6%BC%AB%E7%95%AB%E6%97%A5%E5%92%8CGO%20%2F%20Gyagu%20Manga%20Biyori%20GO%5D%5B03-07%5D%5B1080p%5D%5B%E7%AE%80%E7%B9%81%E4%B8%AD%E6%96%87%5D&tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce",
                                "rclone", "E:\\Download", "od:/Test", 300);
                Thread thread = new TaskUtil();
                thread.start();
        }
}