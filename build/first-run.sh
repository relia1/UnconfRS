docker run \
  -v unconfrs-data:/var/lib/postgresql/data \
  -e UNCONFERENCE_PASSWORD=rc2025-ferris \
  -e ADMIN_EMAIL=bart.massey@gmail.com \
  -e ADMIN_PASSWORD=movhvsllgfqk1X! \
  -p 127.0.0.1:3039:3039 \
  unconfrs-single2
