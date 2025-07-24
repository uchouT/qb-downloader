package moe.uchout.qbdownloader.exception;

public class OverLimitException extends Exception {
    public OverLimitException(String message) {
        super(message);
    }

    public OverLimitException(String message, Throwable cause) {
        super(message, cause);
    }

    public OverLimitException(Throwable cause) {
        super(cause);
    }
}
