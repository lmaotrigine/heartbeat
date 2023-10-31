variable "TAG" {
  default = "latest"
}

variable "RELEASE" {
  default = ""
}

variable "IMAGE_NAME" {
  default = "ghcr.io/lmaotrigine/heartbeat"
}

group "default" {
  targets = ["server"]
}

target "server" {
  tags = ["${IMAGE_NAME}:${TAG}", "${IMAGE_NAME}:latest", notequal("", RELEASE) ? "${IMAGE_NAME}:${RELEASE}" : ""]
}
