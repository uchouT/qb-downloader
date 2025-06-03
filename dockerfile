FROM openjdk:17-jdk-slim
VOLUME /config
COPY target/qbdownloader.jar /usr/app/qbdownloader.jar
WORKDIR /usr/app
ENV PUID=0 PGID=0 UMASK=022
ENV PORT=7845 CONFIG=/config