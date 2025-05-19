package moe.util;

import org.junit.jupiter.api.Test;

import moe.uchout.qbdownloader.util.RcloneUtil;

public class UtilTest {

    @Test
    public static void main(String[] args) {
        RcloneUtil.copy("E:\\Download\\Compressed\\[GBCD-0001] Innocent Story", "od:/uploadtest3/");
    }
}