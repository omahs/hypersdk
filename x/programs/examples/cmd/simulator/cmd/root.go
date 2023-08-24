package cmd

import (
	"github.com/spf13/cobra"

	"github.com/ava-labs/avalanchego/database"
	"github.com/ava-labs/avalanchego/utils/logging"
	"github.com/ava-labs/hypersdk/crypto/ed25519"
	"github.com/ava-labs/hypersdk/pebble"
	"github.com/ava-labs/hypersdk/utils"
	xutils "github.com/ava-labs/hypersdk/x/programs/utils"
)

const (
	defaultDatabase = ".simulator"
)

func init() {
	cobra.EnablePrefixMatching = true
	rootCmd.AddCommand(
		programCmd,
		keyCmd,
	)

	programCmd.AddCommand(
		programCreateCmd,
		programInvokeCmd,
	)

	keyCmd.AddCommand(
		genKeyCmd,
	)

	rootCmd.PersistentFlags().StringVar(
		&dbPath,
		"database",
		defaultDatabase,
		"path to database (will create if missing)",
	)

	rootCmd.PersistentFlags().StringVar(
		&callerAddress,
		"address",
		"",
		"address of caller",
	)

	rootCmd.PersistentPreRunE = func(*cobra.Command, []string) (err error) {
		log = xutils.NewLoggerWithLogLevel(logging.Debug)
		db, _, err = pebble.New(dbPath, pebble.NewDefaultConfig())
		if err != nil {
			return err
		}
		utils.Outf("{{yellow}}database:{{/}} %s\n", dbPath)
		return nil
	}

	rootCmd.PersistentPostRunE = func(*cobra.Command, []string) error {
		return db.Close()
	}

	programCreateCmd.PersistentFlags().StringVar(
		&functions,
		"functions",
		"",
		"comma separated list of function names",
	)

	programInvokeCmd.PersistentFlags().StringVar(
		&programID,
		"id",
		"",
		"id of the program",
	)

	programInvokeCmd.PersistentFlags().StringVar(
		&functionName,
		"function",
		"",
		"name of the function to invoke",
	)

	programCreateCmd.PersistentFlags().StringVar(
		&params,
		"params",
		"",
		"comma separated list of params to pass to the function",
	)
}

var (
	callerAddress string
	pubKey        ed25519.PublicKey
	programID     string
	functionName  string
	dbPath        string
	params        string
	db            database.Database
	log           logging.Logger
	functions     string
	rootCmd       = &cobra.Command{
		Use:   "simulator",
		Short: "HyperSDK program simulator",
	}
)

func Execute() error {
	return rootCmd.Execute()
}
