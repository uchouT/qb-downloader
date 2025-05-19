package moe.uchout.qbdownloader.util;

import org.junit.jupiter.api.Test;
import java.util.*;
import moe.uchout.qbdownloader.entity.*;

public class UtilTest {

    @Test
    public static void main(String[] args) {
        // ConfigUtil.sync();
        ConfigUtil.load();
        System.out.println(ConfigUtil.CONFIG.getQbHost());
        System.out.println(ConfigUtil.CONFIG.getQbUsername());
        System.out.println(ConfigUtil.CONFIG.getQbPassword());
        System.out.println(ConfigUtil.CONFIG.getAlistHost());
        System.out.println(ConfigUtil.CONFIG.getAlistToken());
        System.out.println(ConfigUtil.CONFIG.getRcloneHost());
        System.out.println(ConfigUtil.CONFIG.getRclonePassword());
        System.out.println(ConfigUtil.CONFIG.getRcloneuserName());
    }
}