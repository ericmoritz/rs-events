FROM node

WORKDIR /code
COPY ./package.json ./package.json

RUN npm install
COPY ./features ./features
COPY ./wait-for-it.sh ./wait-for-it.sh
