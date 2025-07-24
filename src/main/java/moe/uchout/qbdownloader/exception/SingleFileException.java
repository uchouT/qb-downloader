package moe.uchout.qbdownloader.exception;
public class SingleFileException extends Exception {
    public SingleFileException(String message) {
        super(message);
    }

    public SingleFileException(String message, Throwable cause) {
        super(message, cause);
    }

    public SingleFileException(Throwable cause) {
        super(cause);
    }
}
