FROM rust
RUN mkdir /app
RUN mkdir /app/source
VOLUME /app
COPY . /app/source
RUN cd /app/source; cargo install --path .
RUN cp /usr/local/cargo/bin/ritlug /ritlug
WORKDIR /app
CMD /ritlug
