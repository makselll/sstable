from locust import HttpUser, task
import random, string

class MyUser(HttpUser):
    @task
    def set(self):
        self.client.post(
            "/set",
            json={
                "key": ''.join(random.choices(string.ascii_uppercase + string.digits, k=10)),
                 "value": ''.join(random.choices(string.ascii_uppercase + string.digits, k=20))
                 },
            headers={"Content-Type": "application/json"}
        )

    @task
    def get(self):
        self.client.post(
            "/get",
            json={
                "key": ''.join(random.choices(string.ascii_uppercase + string.digits, k=10)),
                 },
            headers={"Content-Type": "application/json"}
        )

    @task
    def delete(self):
        self.client.delete(
            "/delete",
            json={
                "key": ''.join(random.choices(string.ascii_uppercase + string.digits, k=10)),
                 },
            headers={"Content-Type": "application/json"}
        )