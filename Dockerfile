FROM node:19 as build

ENV NODE_ENV=production 

WORKDIR /app

COPY package*.json ./

RUN npm install

COPY *.config* ./
COPY ./src ./src
COPY ./static ./static

ENV VITE_MOONRAKER_API=http://192.168.40.6/
ENV VITE_MOONRAKER_WEBSOCKET=ws://192.168.40.6/websocket
ENV HOST=0.0.0.0
ENV PORT=3000

RUN npm run build

FROM node:19-alpine

WORKDIR /app
COPY --from=build /app .



EXPOSE 3000

CMD npm run prod-node
#CMD npm run preview 