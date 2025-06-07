FROM eclipse-temurin:17-jre-alpine

RUN apk add --no-cache curl tzdata && \
    mkdir -p /config && \
    chmod 755 /config

WORKDIR /usr/app

COPY target/qb-downloader.jar qb-downloader.jar

ENV PUID=0 PGID=0 UMASK=022
ENV PORT=7845 CONFIG=/config

VOLUME /config

EXPOSE 7845

HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:${PORT}/ || exit 1

CMD ["java", "-XX:+UseContainerSupport", "-XX:MaxRAMPercentage=80.0", "-Djava.security.egd=file:/dev/./urandom", "-jar", "qb-downloader.jar"]
