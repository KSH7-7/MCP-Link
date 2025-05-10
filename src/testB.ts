import { testMessage, TestClassA } from "./testA.js"

console.log(testMessage)
const instanceA = new TestClassA()
console.log(instanceA.greet())

export function runTestB() {
  console.log("runTestB executed")
  return testMessage
}
