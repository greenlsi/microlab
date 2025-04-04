# microlab
MICROLAB: a virtual lab for learning about microcontrollers



sudo chown -R inesvj:inesvj /home/inesvj/TFG/microlab/interfaz


desde TFG: ./qemu_new/build/qemu-system-arm     -cpu cortex-m4     -machine netduinoplus2     -nographic     -semihosting-config enable=on,target=native     -monitor stdio     -serial null     -qtest tcp:localhost:3000     -kernel ./prueba/test_v1.elf