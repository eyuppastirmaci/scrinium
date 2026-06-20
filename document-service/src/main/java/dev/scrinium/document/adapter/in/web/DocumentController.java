package dev.scrinium.document.adapter.in.web;

import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.port.in.UploadDocument;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.multipart.MultipartFile;

import java.util.UUID;

@RestController
@RequestMapping("/documents")
public class DocumentController {

    private final UploadDocument uploadDocument;
    private final DocumentUploadRequestMapper uploadRequestMapper;

    public DocumentController(UploadDocument uploadDocument,
                              DocumentUploadRequestMapper uploadRequestMapper) {
        this.uploadDocument = uploadDocument;
        this.uploadRequestMapper = uploadRequestMapper;
    }

    @PostMapping(consumes = "multipart/form-data")
    public ResponseEntity<UploadResponse> upload(
            @RequestParam("file") MultipartFile file
    ) {
        Document document = uploadDocument.upload(uploadRequestMapper.toCommand(file));

        return ResponseEntity.accepted()
                .body(new UploadResponse(document.id(), document.status().name()));
    }

    public record UploadResponse(UUID id, String status) {}
}
