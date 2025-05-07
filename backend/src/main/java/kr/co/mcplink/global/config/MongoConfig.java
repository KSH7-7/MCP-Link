package kr.co.mcplink.global.config;

import com.mongodb.client.MongoClient;
import com.mongodb.client.MongoClients;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.data.mongodb.core.MongoTemplate;

@Configuration
public class MongoConfig {

    @Value("${spring.data.mongodb.host}")
    private String mongoHost;
    @Value("${spring.data.mongodb.port}")
    private int mongoPort;
    @Value("${spring.data.mongodb.database}")
    private String mongoDb;
    @Value("${spring.data.mongodb.username}")
    private String mongoUser;
    @Value("${spring.data.mongodb.password}")
    private String mongoPassword;

    @Bean
    public MongoClient mongoClient() {
        String uri = String.format(
                "mongodb://%s:%s@%s:%d/%s?authSource=%s",
                mongoUser,
                mongoPassword,
                mongoHost,
                mongoPort,
                mongoDb,
                mongoDb
        );
        return MongoClients.create(uri);
    }

    @Bean
    public MongoTemplate mongoTemplate(MongoClient client) {
        return new MongoTemplate(client, mongoDb);
    }
}
