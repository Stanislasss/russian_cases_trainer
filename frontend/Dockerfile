# dev: Vite dev server with hot reload (source is bind-mounted by docker-compose).
FROM node:26-slim AS dev
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
EXPOSE 5173
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]

FROM dev AS build
RUN npm run build

# prod: static build served by nginx, which also proxies /api to the backend.
FROM nginx:1-alpine AS prod
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html
