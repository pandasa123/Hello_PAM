using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Diagnostics;
using System.Threading.Tasks;
using Windows.UI.Popups;
using Windows.Security.Credentials;
using Windows.Security.Cryptography;
using Windows.Security.Cryptography.Core;

namespace HelloAuth
{
    class Program
    {
        static int CredentialStatusToExitCode(KeyCredentialStatus status)
        {
            return 171 + (int)status; // Avoid reserved exit codes of UNIX
        }

        static string ExitCodeToMessage(int code, string key_name)
        {
            switch (code)
            {
                case 0:
                    return "Success";
                case 170:
                    return "Windows Hello is not supported in this device";
                case 171:
                    return "The credential already exists. Creation failed";
                case 172:
                    return "The credential '" + key_name + "' does not exist";
                case 173:
                    return "The Windows Hello security device is locked";
                case 175:
                    return "Unknown error";
                case 176:
                    return "The user cancelled";
                case 177:
                    return "The user prefers to enter password. Aborted";
                default:
                    return "Unknown internal error";
            }
        }

        static async Task<(int err, byte[] sig)> VerifyUser(string key_name, string contentToSign)
        {
            if (await KeyCredentialManager.IsSupportedAsync() == false)
            {
                await (new MessageDialog("KeyCredentialManager not supported")).ShowAsync();
                return (170, null);
            }

            var key = await KeyCredentialManager.OpenAsync(key_name);
            if (key.Status != KeyCredentialStatus.Success)
            {
                return (CredentialStatusToExitCode(key.Status), null);
            }

            var buf = CryptographicBuffer.ConvertStringToBinary(contentToSign, BinaryStringEncoding.Utf8);
            var signRes = await key.Credential.RequestSignAsync(buf);
            if (signRes.Status != KeyCredentialStatus.Success)
            {
                return (CredentialStatusToExitCode(key.Status), null);
            }

            byte[] sig;
            CryptographicBuffer.CopyToByteArray(signRes.Result, out sig);
            return (0, sig);
        }

        static void Main(string[] args)
        {
            if (args.Length == 0)
            {
                Console.WriteLine("Usage: HelloAuth.exe credential_key_name");
                Environment.Exit(1);
            }

            var verifyResult = VerifyUser(args[0], Console.In.ReadToEnd()).Result;
            if (verifyResult.err > 0)
            {
                Console.WriteLine(ExitCodeToMessage(verifyResult.err, args[0]));
                Environment.Exit(verifyResult.err);
            }

            var stdout = Console.OpenStandardOutput();
            stdout.Write(verifyResult.sig, 0, verifyResult.sig.Length);
        }
    }
}
