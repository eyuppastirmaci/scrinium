package dev.scrinium.document.adapter.in.web;

import dev.scrinium.document.adapter.in.web.exception.InvalidUploadRequestException;
import dev.scrinium.document.adapter.out.storage.exception.DocumentStorageException;
import dev.scrinium.document.domain.exception.InvalidDocumentException;
import org.springframework.http.HttpStatus;
import org.springframework.http.ProblemDetail;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.RestControllerAdvice;

@RestControllerAdvice
public class DocumentExceptionHandler {

    @ExceptionHandler(InvalidDocumentException.class)
    public ProblemDetail handleInvalidDocument(InvalidDocumentException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.BAD_REQUEST, ex.getMessage()
        );

        problem.setTitle("Invalid document");

        return problem;
    }

    @ExceptionHandler(InvalidUploadRequestException.class)
    public ProblemDetail handleInvalidUploadRequest(InvalidUploadRequestException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.BAD_REQUEST, ex.getMessage()
        );

        problem.setTitle("Invalid upload request");

        return problem;
    }

    @ExceptionHandler(DocumentStorageException.class)
    public ProblemDetail handleDocumentStorage(DocumentStorageException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.SERVICE_UNAVAILABLE, "Document storage is temporarily unavailable"
        );

        problem.setTitle("Document storage unavailable");

        return problem;
    }
}
