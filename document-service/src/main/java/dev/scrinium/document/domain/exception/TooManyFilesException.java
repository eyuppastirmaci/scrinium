package dev.scrinium.document.domain.exception;

public class TooManyFilesException extends DomainException {
    public TooManyFilesException(int actual, int max) {
        super("Too many files in a single request: " + actual + ". Maximum allowed: " + max);
    }
}
