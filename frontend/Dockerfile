FROM node:24-slim AS build
ENV NODE_OPTIONS=--dns-result-order=ipv4first
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts
COPY . .
RUN npx panda codegen && npm run build

FROM nginxinc/nginx-unprivileged:1.29-alpine
COPY nginx.conf /etc/nginx/templates/default.conf.template
COPY security-headers.conf /etc/nginx/snippets/security-headers.conf
COPY --from=build /app/dist/client /usr/share/nginx/html
EXPOSE 8080
