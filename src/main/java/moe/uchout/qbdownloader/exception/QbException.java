package moe.uchout.qbdownloader.exception;

public class QbException extends Exception {
    public QbException(String message) {
        super(message);
    }

    public QbException(String message, Throwable cause) {
        super(message, cause);
    }

    public QbException(Throwable cause) {
        super(cause);
    }
}