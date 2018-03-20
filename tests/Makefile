# This builds and runs the test cluster, the command ends when the cucumber-js command
# ends. The exit code for cucumber-js will be passed back to make.
bdd:
	docker-compose up --build --exit-code-from bdd

# This keeps the cluster running for testing purposes
up:
	docker-compose up --build
