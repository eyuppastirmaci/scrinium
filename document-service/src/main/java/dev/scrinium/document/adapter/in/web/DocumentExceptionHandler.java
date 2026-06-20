package dev.scrinium.document.adapter.in.web;

import dev.scrinium.document.common.FormatUtils;
import dev.scrinium.document.adapter.in.web.exception.InvalidUploadRequestException;
import dev.scrinium.document.adapter.out.storage.exception.DocumentStorageException;
import dev.scrinium.document.domain.exception.DocumentNotFoundException;
import dev.scrinium.document.domain.exception.DuplicateDocumentException;
import dev.scrinium.document.domain.exception.InvalidDocumentException;
import dev.scrinium.document.domain.exception.TooManyFilesException;
import dev.scrinium.document.domain.exception.UnsupportedFileTypeException;
import org.springframework.http.HttpStatus;
import org.springframework.http.ProblemDetail;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.RestControllerAdvice;
import org.springframework.web.method.annotation.MethodArgumentTypeMismatchException;
import org.springframework.web.multipart.MaxUploadSizeExceededException;

@RestControllerAdvice
public class DocumentExceptionHandler {

    // 400 Bad Request: the number of files in the request exceeds the configured maximum.
    @ExceptionHandler(TooManyFilesException.class)
    public ProblemDetail handleTooManyFiles(TooManyFilesException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.BAD_REQUEST, ex.getMessage()
        );

        problem.setTitle("Too many files");

        return problem;
    }

    // 409 Conflict: a non-deleted document with the same SHA-256 content hash already exists.
    @ExceptionHandler(DuplicateDocumentException.class)
    public ProblemDetail handleDuplicateDocument(DuplicateDocumentException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.CONFLICT, ex.getMessage()
        );

        problem.setTitle("Duplicate document");
        problem.setProperty("existingDocumentId", ex.existingDocumentId());

        return problem;
    }

    // 404 Not Found: no accessible document exists with the given id.
    @ExceptionHandler(DocumentNotFoundException.class)
    public ProblemDetail handleDocumentNotFound(DocumentNotFoundException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.NOT_FOUND, ex.getMessage()
        );

        problem.setTitle("Document not found");

        return problem;
    }

    // 400 Bad Request: the document violates a domain invariant (e.g. blank file name, non-positive size).
    @ExceptionHandler(InvalidDocumentException.class)
    public ProblemDetail handleInvalidDocument(InvalidDocumentException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.BAD_REQUEST, ex.getMessage()
        );

        problem.setTitle("Invalid document");

        return problem;
    }

    // 400 Bad Request: the uploaded file could not be read or the request is malformed.
    @ExceptionHandler(InvalidUploadRequestException.class)
    public ProblemDetail handleInvalidUploadRequest(InvalidUploadRequestException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.BAD_REQUEST, ex.getMessage()
        );

        problem.setTitle("Invalid upload request");

        return problem;
    }

    // 415 Unsupported Media Type: the file's content type is not in the supported list.
    @ExceptionHandler(UnsupportedFileTypeException.class)
    public ProblemDetail handleUnsupportedFileType(UnsupportedFileTypeException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.UNSUPPORTED_MEDIA_TYPE, ex.getMessage()
        );

        problem.setTitle("Unsupported file type");

        return problem;
    }

    // 413 Content Too Large: the uploaded file exceeds the configured size limit.
    @ExceptionHandler(MaxUploadSizeExceededException.class)
    public ProblemDetail handleMaxUploadSize(MaxUploadSizeExceededException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.CONTENT_TOO_LARGE,
                "File exceeds the maximum upload size of " + FormatUtils.toMegabytes(ex.getMaxUploadSize())
        );

        problem.setTitle("File too large");

        return problem;
    }

    // 400 Bad Request: a path or query parameter has an invalid type (e.g. non-UUID document id).
    @ExceptionHandler(MethodArgumentTypeMismatchException.class)
    public ProblemDetail handleTypeMismatch(MethodArgumentTypeMismatchException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.BAD_REQUEST, "Invalid value for parameter '" + ex.getName() + "': " + ex.getValue()
        );

        problem.setTitle("Invalid request parameter");

        return problem;
    }

    // 503 Service Unavailable: MinIO or the storage backend is unreachable.
    @ExceptionHandler(DocumentStorageException.class)
    public ProblemDetail handleDocumentStorage(DocumentStorageException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.SERVICE_UNAVAILABLE, "Document storage is temporarily unavailable"
        );

        problem.setTitle("Document storage unavailable");

        return problem;
    }
}
