variable "TAG" {
  default = "latest"
}

variable "IMAGE_NAME" {
  default = "ghcr.io/lmaotrigine/heartbeat"
}

group "default" {
  targets = ["server"]
}

target "server" {
  tags = ["${IMAGE_NAME}:${TAG}", "${IMAGE_NAME}:latest"]
}
