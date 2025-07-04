# practicallypain

<<<<<<< HEAD
=======
This application requires user to login to access any services:
Username: admin
Password: admin123

Username: user1
Password: user123

>>>>>>> aeb901d86534c7be2834f24fbce6edd25a8ed6c3
IF USING COMMAND LINE FOR TESTING:

COMMAND TO LOGIN TO ADMIN ACCOUNT
curl -X POST http://127.0.0.1:8080/login  -H "Content-Type: application/json"  -d '{"username":"admin","password":"admin123"}' 

Once login to Account, Grab the AuthToken given and save it

COMMAND TO REGISTER USER (NEED LOGIN WITH ADMIN FIRST)
curl -X POST http://127.0.0.1:8080/admin/register \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer INSERT_AUTH_TOKEN_HERE" \
  -d '{"username": "user2", "password": "user123"}'


IF USING WEB PAGE:
USE http://127.0.0.1:5000/login to login FIRST prior to other pages
