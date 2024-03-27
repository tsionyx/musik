# https://github.com/GoogleContainerTools/distroless
# TODO: FROM gcr.io/distroless/cc-debian12:latest-arm64
FROM alpine

RUN mkdir -p /opt/bin/
COPY target/release/examples/hsom-exercises /opt/bin/

RUN apk add libgcc gcompat
ENTRYPOINT [ "/opt/bin/hsom-exercises" ]
