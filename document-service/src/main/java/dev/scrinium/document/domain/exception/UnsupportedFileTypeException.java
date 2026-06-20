package dev.scrinium.document.domain.exception;

public class UnsupportedFileTypeException extends DomainException {
    public UnsupportedFileTypeException(String contentType) {
        super("Unsupported file type: " + contentType
                + ". Supported types: PDF, JPEG, PNG, WebP, TIFF");
    }
}
