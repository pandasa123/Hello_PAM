using System;
using System.IO;
using System.Threading.Tasks;
using Windows.Security.Credentials;
using Windows.Security.Cryptography;
using Windows.Storage.Streams;

namespace CredentialCreator
{
    class Program
    {
        static async Task<(int err, string publicKey)> CreatePublicKey(string key_name)
        {
            int err;
            var createRequest = await KeyCredentialManager.RequestCreateAsync(key_name, KeyCredentialCreationOption.FailIfExists);
            IBuffer publicKey;
            if (createRequest.Status == KeyCredentialStatus.CredentialAlreadyExists)
            {
                var existing = await KeyCredentialManager.OpenAsync(key_name);
                if (existing.Status != KeyCredentialStatus.Success)
                {
                    return (1, null);
                }
                err = 170;
                publicKey = existing.Credential.RetrievePublicKey();
            }
            else if (createRequest.Status != KeyCredentialStatus.Success)
            {
                return (1, null);
            }
            else {
                err = 0;
                publicKey = createRequest.Credential.RetrievePublicKey();
            }
            var pem = String.Format("-----BEGIN PUBLIC KEY-----\n{0}\n-----END PUBLIC KEY-----\n", CryptographicBuffer.EncodeToBase64String(publicKey));
            return (err, pem);
        }

        static void exit(int code, bool needPrompt)
        {
            if (needPrompt)
            {
                Console.WriteLine("Hit Enter key to terminate...");
                Console.ReadLine();
            }
            Environment.Exit(code);
        }

        static void Main(string[] args)
        {
            string key_name;
            foreach (var arg in args)
            {
                if (arg == "-h" || arg == "/?")
                {
                    Console.WriteLine("Usage: CredentialCreator.exe [key_name]");
                    Console.WriteLine("This program creates a KeyCredential of Windows Hello, and save it to a file named 'key_name.pem'.");
                    Console.WriteLine("If key_name is not given, the prompt to ask the name will be shown.");
                    return;
                }
            }

            bool needsPrompt = args.Length == 0;

            if (needsPrompt)
            {
                Console.WriteLine("Input the name of the new KeyCredential");
                Console.Write("Name: ");
                key_name = Console.ReadLine();
            }
            else
            {
                key_name = args[0];
            }

            var res = CreatePublicKey(key_name).Result;
            if (res.err == 170)
            {
                Console.WriteLine("Error: The key already exists. Outputting The existing public key.");
            }
            else if (res.err > 0) {
                Console.WriteLine("Error: Key creation failed due to some error");
                exit(res.err, needsPrompt);
            }

            File.WriteAllText(String.Format("{0}.pem", key_name), res.publicKey);
            Console.WriteLine(String.Format("Done. The public credential key is written in '{0}.pem'", key_name));
            exit(res.err, needsPrompt);
        }
    }
}
