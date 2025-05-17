package uchout.qbdownloader.util;

import org.junit.jupiter.api.Test;
import java.util.*;
import uchout.qbdownloader.entity.*;

public class QbUtilTest {
    static String hash = "2896014fcadfb8d5e81689f9482b4226be664aea";

    @Test
    public static void main(String[] args) {
        Config config = new Config();
        QbUtil.login(config);
        QbUtil.setPrio(hash, 1, List.of(48,49));
        QbUtil.recheck(hash);
    }
}