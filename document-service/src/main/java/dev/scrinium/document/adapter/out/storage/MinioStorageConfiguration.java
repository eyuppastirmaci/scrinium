package dev.scrinium.document.adapter.out.storage;

import io.minio.MinioClient;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class MinioStorageConfiguration {

    @Bean
    MinioClient minioClient(
            @Value("${scrinium.storage.endpoint}") String endpoint,
            @Value("${scrinium.storage.access-key}") String accessKey,
            @Value("${scrinium.storage.secret-key}") String secretKey
    ) {
        return MinioClient.builder()
                .endpoint(endpoint)
                .credentials(accessKey, secretKey)
                .build();
    }
}
