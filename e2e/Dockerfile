ARG APP_HOME=/opt/app
ARG BUILD_HOME=/opt/build
ARG CARGO_PIPE_TAG=0.1.3
ARG NAME=app
ARG OUTPUT=run

FROM pipebase/cargo-pipe:${CARGO_PIPE_TAG} as builder
ARG NAME
ARG OUTPUT
ARG TEST
ARG BUILD_HOME
# COPY PROJECT
COPY pipebase pipebase
# BUILD APP
RUN ./build.sh -d pipebase/e2e/${TEST} -m pipe.yml -n ${NAME} -o ${OUTPUT}

# SETUP RUNTIME IMAGE
FROM debian:buster-slim
RUN groupadd pipebase --gid 10000 && \
    useradd pipebase --uid 10000 --gid 10000

ARG APP_HOME
ARG BUILD_HOME
ARG OUTPUT
ARG TEST
ENV APP_HOME=${APP_HOME}
ENV RUN=${OUTPUT}
WORKDIR ${APP_HOME}
# COPY APP BINARY AND PIPE CONFIG
COPY --from=builder ${BUILD_HOME}/${OUTPUT} ${APP_HOME}/${OUTPUT}
COPY ${TEST}/catalogs ${APP_HOME}/catalogs

# RUN APP AS PIPEBASE
RUN chown -R "pipebase:pipebase" ${APP_HOME}
USER pipebase
CMD ["sh", "-c", "${APP_HOME}/${RUN}"]