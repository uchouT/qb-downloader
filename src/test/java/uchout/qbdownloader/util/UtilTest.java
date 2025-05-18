package uchout.qbdownloader.util;

import org.junit.jupiter.api.Test;

public class UtilTest {

    @Test
    public static void main(String[] args) {
        RcloneUtil.copy("E:\\Download\\Compressed\\[GBCD-0001] Innocent Story", "od:/uploadtest3/");
    }
}