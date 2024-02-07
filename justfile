dockerbuild:
    docker build -t daicanglong/starrail-dictionary-backend:latest . --platform linux/x86_64

dockerpush:
    docker push daicanglong/starrail-dictionary-backend:latest