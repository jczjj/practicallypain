# practicallypain

COMMAND TO LOGIN TO ADMIN ACCOUNT
curl -X POST http://127.0.0.1:8080/login  -H "Content-Type: application/json"  -d '{"username":"admin","password":"admin123"}' 

COMMAND TO REGISTER USER (NEED LOGIN WITH PRIOR FIRST)
curl -X POST http://127.0.0.1:8080/admin/register \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbiIsImlzX2FkbWluIjp0cnVlLCJleHAiOjE3NTE2MzY1MzV9.ak9O430kYeEIJ7ub8uDd1rjuU9pNxwBn_jImrvby7KU" \
  -d '{"username": "user2", "password": "user123"}'
