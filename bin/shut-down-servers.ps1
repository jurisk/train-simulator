cd terraform
terraform destroy -target module.proxy -auto-approve
terraform destroy -target module.game_server -auto-approve
cd ..
