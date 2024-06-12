variable "gcp_project" {
  description = "The Google Cloud project ID"
  type        = string
  default     = "train-simulator-gcp"
}

variable "gcp_region" {
  description = "The Google Cloud region"
  type        = string
  default     = "us-east1-c"
}