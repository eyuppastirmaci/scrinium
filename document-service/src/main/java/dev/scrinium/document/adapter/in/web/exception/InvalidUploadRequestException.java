package dev.scrinium.document.adapter.in.web.exception;

public class InvalidUploadRequestException extends RuntimeException {
    public InvalidUploadRequestException(String message, Throwable cause) {
        super(message, cause);
    }
}
