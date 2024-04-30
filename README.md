# 概要

Rust と BigQuery を使った API を Cloud Run 上で実行するアプリケーション

## local から接続

```sh
gcloud run services proxy api --project cloud-run-rust-test
```

## deploy

1. build

   ```sh
     docker build . -t asia-northeast1-docker.pkg.dev/cloud-run-rust-test/cloud-run-rust-test/api:latest
   ```

1. push

   ```sh
     docker push asia-northeast1-docker.pkg.dev/cloud-run-rust-test/cloud-run-rust-test/api:latest
   ```

1. deploy

   ```sh
     gcloud run deploy api \
       --image asia-northeast1-docker.pkg.dev/cloud-run-rust-test/cloud-run-rust-test/api:latest \
       --project=cloud-run-rust-test \
       --port=8080 \
       --region=asia-northeast1 \
       --min-instances=0 \
       --max-instances=50 \
       --memory=512Mi \
       --cpu=1 \
       --no-allow-unauthenticated
   ```
