package main

import (
	"github.com/gin-gonic/gin"
)

func main() {
	r := gin.Default()

	// Example route
	r.GET("/ping", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"message": "pong",
		})
	})

	// Start the server
	r.Run(":8080") // Listen and serve on localhost:8080
}
