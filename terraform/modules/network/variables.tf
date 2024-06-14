variable "gcp_project" {
  description = "The Google Cloud project ID"
  type        = string
}

variable "gcp_region" {
  description = "The Google Cloud region"
  type        = string
}

variable "service_port" {
  description = "The port the service listens on"
  type        = number
}