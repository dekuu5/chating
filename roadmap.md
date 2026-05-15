# ROADMAP


# phase 1 DONE
implement a cli pasring with two sub commands
``` bash
chating listen --port 1230
```

``` bash
chating connect --ip 192.168.1.12 --port 1230 
```

# phase 2
implement a tcp socket connection with a way to print on each party sending


# phase 3
//todo
check if there is the necessary public key and private key in the current directory 

if not gen using openssl
if use use them to to start the chatting 
also genretae a random AES key 
# phase 4 


KEY EXCHANGE
1. B sends pub_B (plaintext)
2. A encrypts pub_A with pub_B → sends to B
   (only real B can unwrap this)
3. B decrypts with pri_B → gets pub_A

MUTUAL AUTHENTICATION (challenge-response)
4. B generates random nonce → encrypts with pub_A → sends
   (only real A can read this)
5. A decrypts nonce with pri_A
6. A signs nonce with pri_A → encrypts signature with pub_B → sends
7. B decrypts with pri_B → verifies signature with pub_A
   (proves A holds pri_A and received the nonce)

AES KEY DELIVERY
8. B generates AES key → encrypts with pub_A → sends
9. A decrypts with pri_A → both now share AES key

CHAT
10. for each message:
    - encrypt message with AES key
    - sign sha256(ciphertext) with sender pri key
    - send [encrypted_message][signature]
    - receiver decrypts with AES
    - receiver verifies signature with sender pub key
    - if signature fails → reject message

# phase 5
after the proces is done we do the chatting between a and b 