package dev.scrinium.document.adapter.out.storage;

import dev.scrinium.document.adapter.out.storage.exception.DocumentStorageException;
import dev.scrinium.document.domain.model.DocumentFile;
import dev.scrinium.document.domain.model.StoredDocument;
import dev.scrinium.document.domain.port.out.DocumentStorage;
import io.minio.GetObjectArgs;
import io.minio.MinioClient;
import io.minio.PutObjectArgs;
import io.minio.RemoveObjectArgs;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import java.io.InputStream;
import java.security.DigestInputStream;
import java.security.MessageDigest;

@Component
public class MinioDocumentStorage implements DocumentStorage {

    private static final String HASH_ALGORITHM = "SHA-256";
    private static final String FALLBACK_FILE_NAME = "document";
    private static final String HEX_BYTE_FORMAT = "%02x";

    private final MinioClient minioClient;
    private final String bucket;

    public MinioDocumentStorage(
            MinioClient minioClient,
            @Value("${scrinium.storage.bucket}") String bucket
    ) {
        this.minioClient = minioClient;
        this.bucket = bucket;
    }

    @Override
    public StoredDocument store(DocumentFile file) {
        String objectKey = objectKeyFor(file);

        try (InputStream input = file.content()) {
            // Wrap the input stream with a digest to compute the SHA-256 hash while streaming to MinIO.
            MessageDigest digest = MessageDigest.getInstance(HASH_ALGORITHM);
            DigestInputStream digestInput = new DigestInputStream(input, digest);

            // Stream the file content to MinIO; the hash is computed on the fly without buffering.
            minioClient.putObject(PutObjectArgs.builder()
                    .bucket(bucket)
                    .object(objectKey)
                    // MinIO accepts -1 as "unknown part size"; the total object size is still provided.
                    .stream(digestInput, file.sizeBytes(), -1L)
                    .contentType(file.contentType())
                    .build());

            return new StoredDocument(
                    objectKey,
                    file.contentType(),
                    file.sizeBytes(),
                    hex(digest.digest())
            );
        } catch (Exception e) {
            throw new DocumentStorageException("Could not store document in MinIO", e);
        }
    }

    @Override
    public InputStream retrieve(String storageObjectKey) {
        try {
            // Stream the file content directly from MinIO without loading it into memory.
            return minioClient.getObject(GetObjectArgs.builder()
                    .bucket(bucket)
                    .object(storageObjectKey)
                    .build());
        } catch (Exception e) {
            throw new DocumentStorageException("Could not retrieve document from MinIO", e);
        }
    }

    @Override
    public void delete(String storageObjectKey) {
        try {
            minioClient.removeObject(RemoveObjectArgs.builder()
                    .bucket(bucket)
                    .object(storageObjectKey)
                    .build());
        } catch (Exception e) {
            throw new DocumentStorageException("Could not delete object from MinIO", e);
        }
    }

    // Builds the stable MinIO object key from the document id and sanitized original file name.
    private String objectKeyFor(DocumentFile file) {
        return "documents/%s/%s".formatted(file.documentId(), safeFileName(file.fileName()));
    }

    // Removes any path components from the client-provided file name and falls back to a safe default.
    private String safeFileName(String fileName) {
        String normalized = fileName.replace('\\', '/');

        int lastSlash = normalized.lastIndexOf('/');

        if (lastSlash >= 0) {
            normalized = normalized.substring(lastSlash + 1);
        }

        if (normalized.isBlank()) {
            return FALLBACK_FILE_NAME;
        }

        return normalized;
    }

    // Converts digest bytes into a lowercase hexadecimal string.
    private String hex(byte[] bytes) {
        // Each byte is rendered as exactly two hexadecimal characters.
        StringBuilder hex = new StringBuilder(bytes.length * 2);

        for (byte value : bytes) {
            hex.append(HEX_BYTE_FORMAT.formatted(value));
        }

        return hex.toString();
    }
}
