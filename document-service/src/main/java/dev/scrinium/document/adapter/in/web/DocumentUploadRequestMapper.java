package dev.scrinium.document.adapter.in.web;

import dev.scrinium.document.adapter.in.web.exception.InvalidUploadRequestException;
import dev.scrinium.document.domain.port.in.UploadDocument;
import org.springframework.stereotype.Component;
import org.springframework.web.multipart.MultipartFile;

import java.io.IOException;
import java.io.InputStream;

@Component
public class DocumentUploadRequestMapper {

    private static final String DEFAULT_CONTENT_TYPE = "application/octet-stream";

    public UploadDocument.Command toCommand(MultipartFile file) {
        return new UploadDocument.Command(
                file.getOriginalFilename(),
                contentTypeOf(file),
                file.getSize(),
                contentOf(file)
        );
    }

    private String contentTypeOf(MultipartFile file) {
        String contentType = file.getContentType();
        if (contentType == null || contentType.isBlank()) {
            return DEFAULT_CONTENT_TYPE;
        }
        return contentType;
    }

    private InputStream contentOf(MultipartFile file) {
        try {
            return file.getInputStream();
        } catch (IOException e) {
            throw new InvalidUploadRequestException("Could not read uploaded file", e);
        }
    }
}
