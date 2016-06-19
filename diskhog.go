package main

import (
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
)

func main() {
}

func backup(source string, dest string) {
	fmt.Printf("backing up %v into %v\n", source, dest)
	contents, err := ioutil.ReadDir(source)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("contents %#v\n", contents)
	for _, item := range contents {
		itemPath := filepath.Join(source, item.Name())
		destFile := filepath.Join(dest, item.Name())
		copyFile(itemPath, destFile)
	}
}

func copyFile(source string, dest string) {
	fmt.Printf("copying %v to : %v\n", source, dest)

	srcFile, err := os.Open(source)
	if err != nil {
		log.Fatal(err)
	}
	defer srcFile.Close()

	destFile, err := os.Create(dest)
	if err != nil {
		log.Fatal(err)
	}
	defer destFile.Close()

	bytesWritten, err := io.Copy(destFile, srcFile)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("%v bytes copied", bytesWritten)
}
