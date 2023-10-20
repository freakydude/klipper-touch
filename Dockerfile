FROM debian:bullseye-slim

WORKDIR /app
VOLUME [ "/app" ]

ENV NODE_MAJOR=18
ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt update -y
RUN apt install -y ca-certificates curl gnupg
RUN mkdir -p /etc/apt/keyrings
RUN curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
RUN echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_MAJOR.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list
RUN apt update -y
RUN apt install -y nodejs libwebkit2gtk-4.0-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

CMD npm install && npm install --save-dev @tauri-apps/cli && npm run tauri build
