# practicallypain

IF USING COMMAND LINE FOR TESTING:

COMMAND TO LOGIN TO ADMIN ACCOUNT
curl -X POST http://127.0.0.1:8080/login  -H "Content-Type: application/json"  -d '{"username":"admin","password":"admin123"}' 

Once login to Account, Grab the AuthToken given and save it

COMMAND TO REGISTER USER (NEED LOGIN WITH ADMIN FIRST)
curl -X POST http://127.0.0.1:8080/admin/register \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer INSERT_AUTH_TOKEN_HERE" \
  -d '{"username": "user2", "password": "user123"}'
