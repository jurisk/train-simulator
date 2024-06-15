variable "gcp_project" {
  description = "The Google Cloud project ID"
  type        = string
}

variable "proxy_server_ip_address" {
  description = "The IP address to associate with the DNS record for the proxy server"
  type        = string
}

variable "game_server_ip_address" {
  description = "The IP address to associate with the DNS record for the game server"
  type        = string
}