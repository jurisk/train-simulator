variable "gcp_project" {
  description = "The Google Cloud project ID"
  type        = string
}

variable "ip_address" {
  description = "The external IP address to associate with the DNS record"
  type        = string
}