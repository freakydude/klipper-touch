FROM node:19 as build

ENV NODE_ENV=production 

WORKDIR /app

COPY package*.json ./

RUN npm install

COPY .env.production ./
COPY *.config* ./
COPY ./src ./src
COPY ./static ./static

RUN npm run build

FROM node:19-alpine

WORKDIR /app
COPY --from=build /app .

ENV HOST=0.0.0.0
EXPOSE 3000

CMD npm run prod-node
#CMD npm run preview 