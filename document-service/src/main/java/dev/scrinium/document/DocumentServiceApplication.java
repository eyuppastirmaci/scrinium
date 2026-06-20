package dev.scrinium.document;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.context.properties.ConfigurationPropertiesScan;

@SpringBootApplication
@ConfigurationPropertiesScan
public class DocumentServiceApplication {

	static void main(String[] args) {
		SpringApplication.run(DocumentServiceApplication.class, args);
	}

}
