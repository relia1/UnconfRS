docker run \
  -v unconfrs-data:/var/lib/postgresql/data \
  -e UNCONFERENCE_PASSWORD=fixme \
  -e ADMIN_EMAIL=fixme@example.com \
  -e ADMIN_PASSWORD=fixme \
  -p 127.0.0.1:3039:3039 \
  unconfrs
