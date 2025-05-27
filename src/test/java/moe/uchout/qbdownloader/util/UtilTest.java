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
                ConfigUtil.CONFIG.setQbHost("http://localhost:8080").setQbUsername("admin").setQbPassword("erzichibaba")
                                .setAlistHost("http://localhost:5244")
                                .setRcloneHost("http://localhost:5572")
                                .setRcloneuserName("uchouT")
                                .setRclonePassword("erzichibaba");
                try {
                        QbUtil.login();
                } catch (Exception e) {
                        e.printStackTrace();
                }
                TaskUtil.addTask(
                                "magnet:?xt=urn:btih:e1a55f736b27302f555a7f660e4298e210b6a2db&dn=My%20Unique%20Skill%20Makes%20Me%20OP%20Even%20at%20Level%201%20v01-08%20%5BVertical%5D%20%5BStick%5D&tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce",
                                "rclone", "E:\\Download", "od:/Test/Rclone", 100);
                Thread thread = new TaskThread();
                thread.start();
        }
}