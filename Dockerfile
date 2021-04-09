# Start with a rust alpine image
FROM rust:alpine3.11
# if needed, install dependencies here
RUN apk add --no-cache musl-dev
# set the workdir and copy the source into it
WORKDIR /app

COPY . .

# do a release build
RUN cargo build --release --locked

# use a plain alpine image, the alpine version needs to match the builder
FROM alpine:3.11
# if needed, install dependencies here
#RUN apk add libseccomp
# copy the binary into the final image
COPY --from=0 /app/target/release/gratico-graphql .
# set the binary as entrypoint
ENTRYPOINT ["/toydb"]
