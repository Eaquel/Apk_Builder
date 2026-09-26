pub const DOCUMENTS: &[(&str, &str)] = &[
    ("en", ENGLISH),
    ("tr", TURKISH),
    ("de", GERMAN),
    ("es", SPANISH),
    ("fr", FRENCH),
    ("it", ITALIAN),
    ("pt", PORTUGUESE),
    ("ru", RUSSIAN),
    ("ar", ARABIC),
    ("zh", CHINESE),
    ("ja", JAPANESE),
    ("ko", KOREAN),
    ("hi", HINDI),
    ("id", INDONESIAN),
    ("pt-BR", BRAZILIAN),
];

const ENGLISH: &str = r#"@@ Terminal Usage
@: Commands The Built-In Terminal Runs On This Device
1- Pwd

Prints the folder the terminal is working in. A new session opens in the project you came from, or in the app's home folder when no project is open.

Example usage:

```
pwd
```

---

2- Ls -La

Lists what the current folder holds, hidden files included, with permissions, size and the time each one last changed.

Example usage:

```
ls -la
```

---

3- Cd

Moves into another folder. cd .. goes one level up and cd ~ returns to the home folder.

Example usage:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Creates a folder, together with any parent folders that do not exist yet.

Example usage:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Copies a file, or with -r a whole folder. The original stays where it is.

Example usage:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Moves a file or folder somewhere else, or renames it in place.

Example usage:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Deletes a file, or with -r a whole folder. Nothing goes to the trash, so read the name once more before pressing Enter.

Example usage:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Show what a file holds: cat prints all of it, head the first lines and tail the last ones.

Example usage:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Searches every file under a folder for a piece of text and prints the file name and line number of each match.

Example usage:

```
grep -rn "onCreate" Java
```

---

10- Find

Finds files by name under the current folder. The name may hold * as a wildcard.

Example usage:

```
find . -name "*.xml"
```

---

11- Wc -L

Counts the lines of a file.

Example usage:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du shows how much space a folder takes; df shows how much room is left on the storage.

Example usage:

```
du -sh .
df -h
```

---

13- Ps / Top

List the processes this app has started. Android shows an app only its own processes, so other apps never appear here.

Example usage:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Read the device's Android version and model, and the kernel it runs.

Example usage:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo prints text or the value of a variable; env lists every variable the shell holds.

Example usage:

```
echo $HOME
env
```

---

16- Tar

Packs files and folders into one compressed archive, or unpacks one.

Example usage:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Prints a file's SHA-256 digest, so you can check it is exactly the file you expect.

Example usage:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear empties the screen; exit ends the shell session.

Example usage:

```
clear
exit
```
"#;

const TURKISH: &str = r#"@@ Terminal Kullanımı
@: Yerleşik Terminalin Bu Cihazda Çalıştırdığı Komutlar
1- Pwd

Terminalin çalıştığı klasörü yazdırır. Yeni bir oturum geldiğiniz projede, açık bir proje yoksa uygulamanın ana klasöründe açılır.

Örnek Kullanım:

```
pwd
```

---

2- Ls -La

Bulunduğunuz klasördekileri gizli dosyalar dahil; izinleri, boyutu ve son değişme zamanıyla listeler.

Örnek Kullanım:

```
ls -la
```

---

3- Cd

Başka bir klasöre geçer. cd .. bir üst klasöre çıkar, cd ~ ana klasöre döner.

Örnek Kullanım:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Bir klasör oluşturur; henüz var olmayan üst klasörleri de birlikte açar.

Örnek Kullanım:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Bir dosyayı, -r ile bütün bir klasörü kopyalar. Asıl dosya yerinde kalır.

Örnek Kullanım:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Bir dosyayı veya klasörü başka bir yere taşır ya da bulunduğu yerde adını değiştirir.

Örnek Kullanım:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Bir dosyayı, -r ile bütün bir klasörü siler. Hiçbir şey çöp kutusuna gitmez; Enter'a basmadan önce adı bir kez daha okuyun.

Örnek Kullanım:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Bir dosyanın içeriğini gösterir: cat tamamını, head ilk satırlarını, tail son satırlarını yazdırır.

Örnek Kullanım:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Bir klasörün altındaki tüm dosyalarda bir metni arar ve her eşleşmenin dosya adını ve satır numarasını yazdırır.

Örnek Kullanım:

```
grep -rn "onCreate" Java
```

---

10- Find

Bulunduğunuz klasörün altında dosyaları adına göre bulur. Adın içinde joker karakter olarak * kullanılabilir.

Örnek Kullanım:

```
find . -name "*.xml"
```

---

11- Wc -L

Bir dosyanın satırlarını sayar.

Örnek Kullanım:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du bir klasörün ne kadar yer kapladığını, df depolamada ne kadar boş alan kaldığını gösterir.

Örnek Kullanım:

```
du -sh .
df -h
```

---

13- Ps / Top

Bu uygulamanın başlattığı süreçleri listeler. Android bir uygulamaya yalnızca kendi süreçlerini gösterir; diğer uygulamalar burada görünmez.

Örnek Kullanım:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Cihazın Android sürümünü, modelini ve çalıştırdığı çekirdeği okur.

Örnek Kullanım:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo bir metni ya da bir değişkenin değerini yazdırır; env kabuğun tuttuğu bütün değişkenleri listeler.

Örnek Kullanım:

```
echo $HOME
env
```

---

16- Tar

Dosyaları ve klasörleri tek bir sıkıştırılmış arşivde toplar ya da bir arşivi açar.

Örnek Kullanım:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Bir dosyanın SHA-256 özetini yazdırır; böylece dosyanın tam olarak beklediğiniz dosya olduğunu doğrulayabilirsiniz.

Örnek Kullanım:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear ekranı temizler; exit kabuk oturumunu kapatır.

Örnek Kullanım:

```
clear
exit
```
"#;

const GERMAN: &str = r#"@@ Terminal-Nutzung
@: Befehle, Die Das Eingebaute Terminal Auf Diesem Gerät Ausführt
1- Pwd

Gibt den Ordner aus, in dem das Terminal arbeitet. Eine neue Sitzung öffnet sich im Projekt, aus dem Sie kommen, oder im Home-Ordner der App, wenn kein Projekt offen ist.

Beispiel:

```
pwd
```

---

2- Ls -La

Listet den Inhalt des aktuellen Ordners einschließlich versteckter Dateien, mit Rechten, Größe und Zeitpunkt der letzten Änderung.

Beispiel:

```
ls -la
```

---

3- Cd

Wechselt in einen anderen Ordner. cd .. geht eine Ebene nach oben, cd ~ zurück in den Home-Ordner.

Beispiel:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Legt einen Ordner an, zusammen mit allen übergeordneten Ordnern, die noch fehlen.

Beispiel:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Kopiert eine Datei oder mit -r einen ganzen Ordner. Das Original bleibt, wo es ist.

Beispiel:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Verschiebt eine Datei oder einen Ordner an einen anderen Ort oder benennt sie an Ort und Stelle um.

Beispiel:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Löscht eine Datei oder mit -r einen ganzen Ordner. Nichts landet im Papierkorb, also lesen Sie den Namen noch einmal, bevor Sie Enter drücken.

Beispiel:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Zeigen den Inhalt einer Datei: cat gibt alles aus, head die ersten Zeilen, tail die letzten.

Beispiel:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Durchsucht alle Dateien unter einem Ordner nach einem Text und gibt Dateiname und Zeilennummer jedes Treffers aus.

Beispiel:

```
grep -rn "onCreate" Java
```

---

10- Find

Findet Dateien unter dem aktuellen Ordner nach ihrem Namen. Der Name darf * als Platzhalter enthalten.

Beispiel:

```
find . -name "*.xml"
```

---

11- Wc -L

Zählt die Zeilen einer Datei.

Beispiel:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du zeigt, wie viel Platz ein Ordner belegt; df zeigt, wie viel Speicher noch frei ist.

Beispiel:

```
du -sh .
df -h
```

---

13- Ps / Top

Listen die Prozesse, die diese App gestartet hat. Android zeigt einer App nur ihre eigenen Prozesse, andere Apps erscheinen hier nie.

Beispiel:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Lesen die Android-Version und das Modell des Geräts sowie den laufenden Kernel.

Beispiel:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo gibt Text oder den Wert einer Variable aus; env listet alle Variablen der Shell.

Beispiel:

```
echo $HOME
env
```

---

16- Tar

Packt Dateien und Ordner in ein komprimiertes Archiv oder entpackt eines.

Beispiel:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Gibt den SHA-256-Hash einer Datei aus, damit Sie prüfen können, dass es genau die erwartete Datei ist.

Beispiel:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear leert den Bildschirm; exit beendet die Shell-Sitzung.

Beispiel:

```
clear
exit
```
"#;

const SPANISH: &str = r#"@@ Uso Del Terminal
@: Comandos Que El Terminal Integrado Ejecuta En Este Dispositivo
1- Pwd

Muestra la carpeta en la que trabaja el terminal. Una sesión nueva se abre en el proyecto del que vienes, o en la carpeta personal de la app si no hay ningún proyecto abierto.

Ejemplo de uso:

```
pwd
```

---

2- Ls -La

Lista el contenido de la carpeta actual, incluidos los archivos ocultos, con permisos, tamaño y la hora del último cambio.

Ejemplo de uso:

```
ls -la
```

---

3- Cd

Entra en otra carpeta. cd .. sube un nivel y cd ~ vuelve a la carpeta personal.

Ejemplo de uso:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Crea una carpeta junto con las carpetas superiores que aún no existen.

Ejemplo de uso:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Copia un archivo, o con -r una carpeta entera. El original se queda donde está.

Ejemplo de uso:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Mueve un archivo o una carpeta a otro lugar, o le cambia el nombre en el mismo sitio.

Ejemplo de uso:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Borra un archivo, o con -r una carpeta entera. Nada va a la papelera, así que lee el nombre una vez más antes de pulsar Intro.

Ejemplo de uso:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Muestran lo que contiene un archivo: cat lo imprime entero, head las primeras líneas y tail las últimas.

Ejemplo de uso:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Busca un texto en todos los archivos de una carpeta e imprime el nombre del archivo y el número de línea de cada coincidencia.

Ejemplo de uso:

```
grep -rn "onCreate" Java
```

---

10- Find

Encuentra archivos por su nombre dentro de la carpeta actual. El nombre puede llevar * como comodín.

Ejemplo de uso:

```
find . -name "*.xml"
```

---

11- Wc -L

Cuenta las líneas de un archivo.

Ejemplo de uso:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du muestra cuánto espacio ocupa una carpeta; df muestra cuánto espacio libre queda en el almacenamiento.

Ejemplo de uso:

```
du -sh .
df -h
```

---

13- Ps / Top

Listan los procesos que ha iniciado esta app. Android solo le muestra a una app sus propios procesos, así que aquí nunca aparecen otras apps.

Ejemplo de uso:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Leen la versión de Android y el modelo del dispositivo, y el kernel que ejecuta.

Ejemplo de uso:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo imprime un texto o el valor de una variable; env lista todas las variables que guarda el shell.

Ejemplo de uso:

```
echo $HOME
env
```

---

16- Tar

Empaqueta archivos y carpetas en un único archivo comprimido, o descomprime uno.

Ejemplo de uso:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Imprime el resumen SHA-256 de un archivo para comprobar que es exactamente el archivo que esperas.

Ejemplo de uso:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear limpia la pantalla; exit cierra la sesión del shell.

Ejemplo de uso:

```
clear
exit
```
"#;

const FRENCH: &str = r#"@@ Utilisation Du Terminal
@: Commandes Que Le Terminal Intégré Exécute Sur Cet Appareil
1- Pwd

Affiche le dossier dans lequel travaille le terminal. Une nouvelle session s’ouvre dans le projet d’où vous venez, ou dans le dossier personnel de l’app si aucun projet n’est ouvert.

Exemple d’utilisation :

```
pwd
```

---

2- Ls -La

Liste le contenu du dossier courant, fichiers cachés compris, avec les droits, la taille et l’heure de la dernière modification.

Exemple d’utilisation :

```
ls -la
```

---

3- Cd

Passe dans un autre dossier. cd .. remonte d’un niveau et cd ~ revient au dossier personnel.

Exemple d’utilisation :

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Crée un dossier, ainsi que les dossiers parents qui n’existent pas encore.

Exemple d’utilisation :

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Copie un fichier, ou avec -r un dossier entier. L’original reste à sa place.

Exemple d’utilisation :

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Déplace un fichier ou un dossier ailleurs, ou le renomme sur place.

Exemple d’utilisation :

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Supprime un fichier, ou avec -r un dossier entier. Rien ne part à la corbeille : relisez le nom avant d’appuyer sur Entrée.

Exemple d’utilisation :

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Affichent le contenu d’un fichier : cat l’imprime en entier, head les premières lignes et tail les dernières.

Exemple d’utilisation :

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Cherche un texte dans tous les fichiers d’un dossier et affiche le nom du fichier et le numéro de ligne de chaque résultat.

Exemple d’utilisation :

```
grep -rn "onCreate" Java
```

---

10- Find

Trouve des fichiers par leur nom sous le dossier courant. Le nom peut contenir * comme joker.

Exemple d’utilisation :

```
find . -name "*.xml"
```

---

11- Wc -L

Compte les lignes d’un fichier.

Exemple d’utilisation :

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du indique la place qu’occupe un dossier ; df indique l’espace encore libre sur le stockage.

Exemple d’utilisation :

```
du -sh .
df -h
```

---

13- Ps / Top

Listent les processus lancés par cette app. Android ne montre à une app que ses propres processus : les autres apps n’apparaissent jamais ici.

Exemple d’utilisation :

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Lisent la version d’Android et le modèle de l’appareil, ainsi que le noyau qui tourne.

Exemple d’utilisation :

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo affiche un texte ou la valeur d’une variable ; env liste toutes les variables du shell.

Exemple d’utilisation :

```
echo $HOME
env
```

---

16- Tar

Regroupe des fichiers et des dossiers dans une archive compressée, ou en décompresse une.

Exemple d’utilisation :

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Affiche l’empreinte SHA-256 d’un fichier, pour vérifier que c’est exactement le fichier attendu.

Exemple d’utilisation :

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear vide l’écran ; exit ferme la session du shell.

Exemple d’utilisation :

```
clear
exit
```
"#;

const ITALIAN: &str = r#"@@ Uso Del Terminale
@: Comandi Che Il Terminale Integrato Esegue Su Questo Dispositivo
1- Pwd

Mostra la cartella in cui lavora il terminale. Una nuova sessione si apre nel progetto da cui arrivi, oppure nella cartella home dell’app se non c’è un progetto aperto.

Esempio d’uso:

```
pwd
```

---

2- Ls -La

Elenca il contenuto della cartella corrente, file nascosti compresi, con permessi, dimensione e ora dell’ultima modifica.

Esempio d’uso:

```
ls -la
```

---

3- Cd

Entra in un’altra cartella. cd .. sale di un livello e cd ~ torna alla cartella home.

Esempio d’uso:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Crea una cartella insieme alle cartelle superiori che ancora non esistono.

Esempio d’uso:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Copia un file, o con -r un’intera cartella. L’originale resta dov’è.

Esempio d’uso:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Sposta un file o una cartella altrove, oppure lo rinomina sul posto.

Esempio d’uso:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Elimina un file, o con -r un’intera cartella. Nulla finisce nel cestino, quindi rileggi il nome prima di premere Invio.

Esempio d’uso:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Mostrano il contenuto di un file: cat lo stampa tutto, head le prime righe e tail le ultime.

Esempio d’uso:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Cerca un testo in tutti i file di una cartella e stampa il nome del file e il numero di riga di ogni corrispondenza.

Esempio d’uso:

```
grep -rn "onCreate" Java
```

---

10- Find

Trova i file per nome sotto la cartella corrente. Il nome può contenere * come carattere jolly.

Esempio d’uso:

```
find . -name "*.xml"
```

---

11- Wc -L

Conta le righe di un file.

Esempio d’uso:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du mostra quanto spazio occupa una cartella; df mostra quanto spazio resta libero nella memoria.

Esempio d’uso:

```
du -sh .
df -h
```

---

13- Ps / Top

Elencano i processi avviati da questa app. Android mostra a un’app solo i suoi processi, quindi le altre app qui non compaiono mai.

Esempio d’uso:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Leggono la versione di Android e il modello del dispositivo, e il kernel in esecuzione.

Esempio d’uso:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo stampa un testo o il valore di una variabile; env elenca tutte le variabili della shell.

Esempio d’uso:

```
echo $HOME
env
```

---

16- Tar

Raccoglie file e cartelle in un unico archivio compresso, oppure ne estrae uno.

Esempio d’uso:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Stampa l’impronta SHA-256 di un file, per verificare che sia esattamente il file che ti aspetti.

Esempio d’uso:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear pulisce lo schermo; exit chiude la sessione della shell.

Esempio d’uso:

```
clear
exit
```
"#;

const PORTUGUESE: &str = r#"@@ Utilização Do Terminal
@: Comandos Que O Terminal Integrado Executa Neste Dispositivo
1- Pwd

Mostra a pasta em que o terminal está a trabalhar. Uma nova sessão abre no projeto de onde veio, ou na pasta pessoal da app quando não há nenhum projeto aberto.

Exemplo de utilização:

```
pwd
```

---

2- Ls -La

Lista o conteúdo da pasta atual, incluindo ficheiros ocultos, com permissões, tamanho e a hora da última alteração.

Exemplo de utilização:

```
ls -la
```

---

3- Cd

Entra noutra pasta. cd .. sobe um nível e cd ~ volta à pasta pessoal.

Exemplo de utilização:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Cria uma pasta, juntamente com as pastas superiores que ainda não existem.

Exemplo de utilização:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Copia um ficheiro, ou com -r uma pasta inteira. O original fica onde está.

Exemplo de utilização:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Move um ficheiro ou uma pasta para outro sítio, ou muda-lhe o nome no mesmo local.

Exemplo de utilização:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Apaga um ficheiro, ou com -r uma pasta inteira. Nada vai para o lixo, por isso leia o nome mais uma vez antes de premir Enter.

Exemplo de utilização:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Mostram o conteúdo de um ficheiro: cat imprime-o todo, head as primeiras linhas e tail as últimas.

Exemplo de utilização:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Procura um texto em todos os ficheiros de uma pasta e imprime o nome do ficheiro e o número da linha de cada ocorrência.

Exemplo de utilização:

```
grep -rn "onCreate" Java
```

---

10- Find

Encontra ficheiros pelo nome dentro da pasta atual. O nome pode conter * como caráter universal.

Exemplo de utilização:

```
find . -name "*.xml"
```

---

11- Wc -L

Conta as linhas de um ficheiro.

Exemplo de utilização:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du mostra quanto espaço ocupa uma pasta; df mostra quanto espaço livre resta no armazenamento.

Exemplo de utilização:

```
du -sh .
df -h
```

---

13- Ps / Top

Listam os processos que esta app iniciou. O Android só mostra a uma app os seus próprios processos, por isso as outras apps nunca aparecem aqui.

Exemplo de utilização:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Leem a versão do Android e o modelo do dispositivo, e o kernel em execução.

Exemplo de utilização:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo imprime um texto ou o valor de uma variável; env lista todas as variáveis da shell.

Exemplo de utilização:

```
echo $HOME
env
```

---

16- Tar

Junta ficheiros e pastas num único arquivo comprimido, ou extrai um.

Exemplo de utilização:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Imprime o resumo SHA-256 de um ficheiro, para confirmar que é exatamente o ficheiro que espera.

Exemplo de utilização:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear limpa o ecrã; exit termina a sessão da shell.

Exemplo de utilização:

```
clear
exit
```
"#;

const RUSSIAN: &str = r#"@@ Использование Терминала
@: Команды, Которые Встроенный Терминал Выполняет На Этом Устройстве
1- Pwd

Показывает папку, в которой работает терминал. Новый сеанс открывается в проекте, из которого вы пришли, или в домашней папке приложения, если проект не открыт.

Пример использования:

```
pwd
```

---

2- Ls -La

Выводит содержимое текущей папки, включая скрытые файлы, с правами, размером и временем последнего изменения.

Пример использования:

```
ls -la
```

---

3- Cd

Переходит в другую папку. cd .. поднимается на уровень выше, cd ~ возвращает в домашнюю папку.

Пример использования:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Создаёт папку вместе со всеми родительскими папками, которых ещё нет.

Пример использования:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Копирует файл, а с -r — целую папку. Оригинал остаётся на месте.

Пример использования:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Перемещает файл или папку в другое место или переименовывает на месте.

Пример использования:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Удаляет файл, а с -r — целую папку. В корзину ничего не попадает, поэтому перечитайте имя, прежде чем нажать Enter.

Пример использования:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Показывают содержимое файла: cat выводит его целиком, head — первые строки, tail — последние.

Пример использования:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Ищет текст во всех файлах папки и выводит имя файла и номер строки каждого совпадения.

Пример использования:

```
grep -rn "onCreate" Java
```

---

10- Find

Находит файлы по имени в текущей папке. В имени можно использовать * как шаблон.

Пример использования:

```
find . -name "*.xml"
```

---

11- Wc -L

Считает строки в файле.

Пример использования:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du показывает, сколько места занимает папка; df — сколько свободного места осталось в хранилище.

Пример использования:

```
du -sh .
df -h
```

---

13- Ps / Top

Показывают процессы, запущенные этим приложением. Android показывает приложению только его собственные процессы, поэтому другие приложения здесь не появляются.

Пример использования:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Показывают версию Android и модель устройства, а также работающее ядро.

Пример использования:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo выводит текст или значение переменной; env показывает все переменные оболочки.

Пример использования:

```
echo $HOME
env
```

---

16- Tar

Упаковывает файлы и папки в один сжатый архив или распаковывает архив.

Пример использования:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Выводит хеш SHA-256 файла, чтобы убедиться, что это именно тот файл, который вы ожидаете.

Пример использования:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear очищает экран; exit завершает сеанс оболочки.

Пример использования:

```
clear
exit
```
"#;

const ARABIC: &str = r#"@@ استخدام الطرفية
@: الأوامر التي تنفذها الطرفية المدمجة على هذا الجهاز
1- Pwd

يطبع المجلد الذي تعمل فيه الطرفية. تُفتح الجلسة الجديدة في المشروع الذي جئت منه، أو في المجلد الرئيسي للتطبيق إذا لم يكن هناك مشروع مفتوح.

مثال على الاستخدام:

```
pwd
```

---

2- Ls -La

يعرض محتويات المجلد الحالي، بما فيها الملفات المخفية، مع الأذونات والحجم ووقت آخر تعديل.

مثال على الاستخدام:

```
ls -la
```

---

3- Cd

ينتقل إلى مجلد آخر. cd .. يصعد مستوى واحدًا، و cd ~ يعود إلى المجلد الرئيسي.

مثال على الاستخدام:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

ينشئ مجلدًا مع كل المجلدات الأعلى التي لم تُنشأ بعد.

مثال على الاستخدام:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

ينسخ ملفًا، أو مجلدًا كاملًا مع -r. يبقى الأصل في مكانه.

مثال على الاستخدام:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

ينقل ملفًا أو مجلدًا إلى مكان آخر، أو يغيّر اسمه في مكانه.

مثال على الاستخدام:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

يحذف ملفًا، أو مجلدًا كاملًا مع -r. لا يذهب شيء إلى سلة المهملات، لذا اقرأ الاسم مرة أخرى قبل الضغط على Enter.

مثال على الاستخدام:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

تعرض محتوى ملف: cat يطبعه كاملًا، و head الأسطر الأولى، و tail الأسطر الأخيرة.

مثال على الاستخدام:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

يبحث عن نص في كل ملفات مجلد ويطبع اسم الملف ورقم السطر لكل تطابق.

مثال على الاستخدام:

```
grep -rn "onCreate" Java
```

---

10- Find

يجد الملفات بالاسم داخل المجلد الحالي. يمكن أن يحتوي الاسم على * كحرف بدل.

مثال على الاستخدام:

```
find . -name "*.xml"
```

---

11- Wc -L

يعدّ أسطر ملف.

مثال على الاستخدام:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du يبيّن المساحة التي يشغلها مجلد، و df يبيّن المساحة الفارغة المتبقية في التخزين.

مثال على الاستخدام:

```
du -sh .
df -h
```

---

13- Ps / Top

تعرض العمليات التي بدأها هذا التطبيق. يُظهر أندرويد لكل تطبيق عملياته فقط، لذلك لا تظهر التطبيقات الأخرى هنا أبدًا.

مثال على الاستخدام:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

تقرأ إصدار أندرويد وطراز الجهاز والنواة التي يعمل بها.

مثال على الاستخدام:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo يطبع نصًا أو قيمة متغير، و env يعرض كل المتغيرات التي تحملها الصدفة.

مثال على الاستخدام:

```
echo $HOME
env
```

---

16- Tar

يجمع الملفات والمجلدات في أرشيف مضغوط واحد، أو يفك أرشيفًا.

مثال على الاستخدام:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

يطبع بصمة SHA-256 لملف، لتتأكد أنه الملف الذي تتوقعه تمامًا.

مثال على الاستخدام:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear يمسح الشاشة، و exit ينهي جلسة الصدفة.

مثال على الاستخدام:

```
clear
exit
```
"#;

const CHINESE: &str = r#"@@ 终端使用
@: 内置终端在本设备上运行的命令
1- Pwd

显示终端当前所在的文件夹。新会话会在你来的项目中打开；如果没有打开的项目，则在应用的主文件夹中打开。

使用示例：

```
pwd
```

---

2- Ls -La

列出当前文件夹的内容（包括隐藏文件），并显示权限、大小和最后修改时间。

使用示例：

```
ls -la
```

---

3- Cd

进入另一个文件夹。cd .. 返回上一级，cd ~ 回到主文件夹。

使用示例：

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

创建一个文件夹，并一起创建尚不存在的上级文件夹。

使用示例：

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

复制一个文件；加上 -r 可复制整个文件夹。原文件保持不变。

使用示例：

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

把文件或文件夹移动到别处，或在原处重命名。

使用示例：

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

删除一个文件；加上 -r 可删除整个文件夹。不会进入回收站，所以按回车前请再看一遍名称。

使用示例：

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

显示文件内容：cat 打印全部，head 打印开头几行，tail 打印最后几行。

使用示例：

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

在一个文件夹下的所有文件中搜索文本，并打印每个匹配项所在的文件名和行号。

使用示例：

```
grep -rn "onCreate" Java
```

---

10- Find

按名称在当前文件夹下查找文件。名称中可以使用 * 作为通配符。

使用示例：

```
find . -name "*.xml"
```

---

11- Wc -L

统计文件的行数。

使用示例：

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du 显示文件夹占用的空间；df 显示存储中剩余的可用空间。

使用示例：

```
du -sh .
df -h
```

---

13- Ps / Top

列出本应用启动的进程。Android 只向应用显示它自己的进程，所以其他应用不会出现在这里。

使用示例：

```
ps -A
top -n 1
```

---

14- Getprop / Uname

读取设备的 Android 版本、型号以及正在运行的内核。

使用示例：

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo 打印文本或变量的值；env 列出 shell 中的所有变量。

使用示例：

```
echo $HOME
env
```

---

16- Tar

把文件和文件夹打包成一个压缩归档，或解压一个归档。

使用示例：

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

打印文件的 SHA-256 摘要，用来确认它正是你期望的那个文件。

使用示例：

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear 清空屏幕；exit 结束 shell 会话。

使用示例：

```
clear
exit
```
"#;

const JAPANESE: &str = r#"@@ ターミナルの使い方
@: 内蔵ターミナルがこの端末で実行するコマンド
1- Pwd

ターミナルが作業しているフォルダを表示します。新しいセッションは移動元のプロジェクトで、プロジェクトが開いていない場合はアプリのホームフォルダで開きます。

使用例：

```
pwd
```

---

2- Ls -La

現在のフォルダの内容を、隠しファイルも含めて、権限・サイズ・最終更新時刻とともに一覧表示します。

使用例：

```
ls -la
```

---

3- Cd

別のフォルダに移動します。cd .. は一つ上へ、cd ~ はホームフォルダへ戻ります。

使用例：

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

フォルダを作成します。まだ存在しない親フォルダもまとめて作成します。

使用例：

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

ファイルをコピーします。-r を付けるとフォルダ全体をコピーします。元のファイルはそのまま残ります。

使用例：

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

ファイルやフォルダを別の場所へ移動するか、その場で名前を変更します。

使用例：

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

ファイルを削除します。-r を付けるとフォルダ全体を削除します。ゴミ箱には入らないので、Enter を押す前に名前をもう一度確認してください。

使用例：

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

ファイルの内容を表示します。cat は全体を、head は先頭の行を、tail は末尾の行を表示します。

使用例：

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

フォルダ内のすべてのファイルからテキストを検索し、一致した箇所ごとにファイル名と行番号を表示します。

使用例：

```
grep -rn "onCreate" Java
```

---

10- Find

現在のフォルダ以下で名前によってファイルを探します。名前にはワイルドカードとして * を使えます。

使用例：

```
find . -name "*.xml"
```

---

11- Wc -L

ファイルの行数を数えます。

使用例：

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du はフォルダが使っている容量を、df はストレージの空き容量を表示します。

使用例：

```
du -sh .
df -h
```

---

13- Ps / Top

このアプリが起動したプロセスを一覧表示します。Android はアプリに自分のプロセスだけを見せるため、他のアプリはここに表示されません。

使用例：

```
ps -A
top -n 1
```

---

14- Getprop / Uname

端末の Android バージョン、モデル、動作中のカーネルを読み取ります。

使用例：

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo はテキストや変数の値を表示し、env はシェルが持つすべての変数を一覧表示します。

使用例：

```
echo $HOME
env
```

---

16- Tar

ファイルやフォルダを一つの圧縮アーカイブにまとめるか、アーカイブを展開します。

使用例：

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

ファイルの SHA-256 ダイジェストを表示し、期待どおりのファイルであることを確認できます。

使用例：

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear は画面を消去し、exit はシェルのセッションを終了します。

使用例：

```
clear
exit
```
"#;

const KOREAN: &str = r#"@@ 터미널 사용법
@: 내장 터미널이 이 기기에서 실행하는 명령어
1- Pwd

터미널이 작업 중인 폴더를 표시합니다. 새 세션은 방금 있던 프로젝트에서 열리고, 열린 프로젝트가 없으면 앱의 홈 폴더에서 열립니다.

사용 예:

```
pwd
```

---

2- Ls -La

현재 폴더의 내용을 숨김 파일까지 포함해 권한, 크기, 마지막 수정 시간과 함께 나열합니다.

사용 예:

```
ls -la
```

---

3- Cd

다른 폴더로 이동합니다. cd .. 는 한 단계 위로, cd ~ 는 홈 폴더로 돌아갑니다.

사용 예:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

폴더를 만들고, 아직 없는 상위 폴더도 함께 만듭니다.

사용 예:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

파일을 복사합니다. -r 을 붙이면 폴더 전체를 복사합니다. 원본은 그대로 남습니다.

사용 예:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

파일이나 폴더를 다른 곳으로 옮기거나 그 자리에서 이름을 바꿉니다.

사용 예:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

파일을 삭제합니다. -r 을 붙이면 폴더 전체를 삭제합니다. 휴지통으로 가지 않으므로 Enter 를 누르기 전에 이름을 한 번 더 확인하세요.

사용 예:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

파일 내용을 보여 줍니다. cat 은 전체를, head 는 앞부분 줄을, tail 은 마지막 줄을 출력합니다.

사용 예:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

폴더 아래의 모든 파일에서 텍스트를 찾고, 일치하는 곳마다 파일 이름과 줄 번호를 출력합니다.

사용 예:

```
grep -rn "onCreate" Java
```

---

10- Find

현재 폴더 아래에서 이름으로 파일을 찾습니다. 이름에 와일드카드로 * 를 쓸 수 있습니다.

사용 예:

```
find . -name "*.xml"
```

---

11- Wc -L

파일의 줄 수를 셉니다.

사용 예:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du 는 폴더가 차지하는 공간을, df 는 저장소에 남은 여유 공간을 보여 줍니다.

사용 예:

```
du -sh .
df -h
```

---

13- Ps / Top

이 앱이 시작한 프로세스를 나열합니다. Android 는 앱에게 자신의 프로세스만 보여 주므로 다른 앱은 여기에 나타나지 않습니다.

사용 예:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

기기의 Android 버전과 모델, 실행 중인 커널을 읽습니다.

사용 예:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo 는 텍스트나 변수 값을 출력하고, env 는 셸이 가진 모든 변수를 나열합니다.

사용 예:

```
echo $HOME
env
```

---

16- Tar

파일과 폴더를 하나의 압축 아카이브로 묶거나 아카이브를 풉니다.

사용 예:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

파일의 SHA-256 다이제스트를 출력해 기대한 파일이 맞는지 확인할 수 있습니다.

사용 예:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear 는 화면을 지우고, exit 는 셸 세션을 끝냅니다.

사용 예:

```
clear
exit
```
"#;

const HINDI: &str = r#"@@ टर्मिनल का उपयोग
@: वे कमांड जिन्हें बिल्ट-इन टर्मिनल इस डिवाइस पर चलाता है
1- Pwd

टर्मिनल जिस फ़ोल्डर में काम कर रहा है, उसे दिखाता है। नया सत्र उस प्रोजेक्ट में खुलता है जहाँ से आप आए हैं, और कोई प्रोजेक्ट खुला न हो तो ऐप के होम फ़ोल्डर में।

उपयोग का उदाहरण:

```
pwd
```

---

2- Ls -La

मौजूदा फ़ोल्डर की सामग्री, छिपी फ़ाइलों सहित, अनुमतियों, आकार और आख़िरी बदलाव के समय के साथ दिखाता है।

उपयोग का उदाहरण:

```
ls -la
```

---

3- Cd

दूसरे फ़ोल्डर में जाता है। cd .. एक स्तर ऊपर जाता है और cd ~ होम फ़ोल्डर पर लौटाता है।

उपयोग का उदाहरण:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

एक फ़ोल्डर बनाता है, साथ में वे ऊपरी फ़ोल्डर भी जो अभी मौजूद नहीं हैं।

उपयोग का उदाहरण:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

एक फ़ाइल कॉपी करता है, या -r के साथ पूरा फ़ोल्डर। मूल वहीं रहता है।

उपयोग का उदाहरण:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

किसी फ़ाइल या फ़ोल्डर को दूसरी जगह ले जाता है, या उसी जगह उसका नाम बदलता है।

उपयोग का उदाहरण:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

एक फ़ाइल हटाता है, या -r के साथ पूरा फ़ोल्डर। कुछ भी ट्रैश में नहीं जाता, इसलिए Enter दबाने से पहले नाम एक बार फिर पढ़ें।

उपयोग का उदाहरण:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

फ़ाइल की सामग्री दिखाते हैं: cat पूरी फ़ाइल, head शुरुआती पंक्तियाँ और tail आख़िरी पंक्तियाँ छापता है।

उपयोग का उदाहरण:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

किसी फ़ोल्डर की सभी फ़ाइलों में टेक्स्ट खोजता है और हर मिलान का फ़ाइल नाम और पंक्ति संख्या छापता है।

उपयोग का उदाहरण:

```
grep -rn "onCreate" Java
```

---

10- Find

मौजूदा फ़ोल्डर के अंदर नाम से फ़ाइलें ढूँढता है। नाम में वाइल्डकार्ड के रूप में * इस्तेमाल हो सकता है।

उपयोग का उदाहरण:

```
find . -name "*.xml"
```

---

11- Wc -L

किसी फ़ाइल की पंक्तियाँ गिनता है।

उपयोग का उदाहरण:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du बताता है कि फ़ोल्डर कितनी जगह लेता है; df बताता है कि स्टोरेज में कितनी खाली जगह बची है।

उपयोग का उदाहरण:

```
du -sh .
df -h
```

---

13- Ps / Top

इस ऐप द्वारा शुरू की गई प्रक्रियाएँ दिखाते हैं। Android किसी ऐप को केवल उसकी अपनी प्रक्रियाएँ दिखाता है, इसलिए दूसरे ऐप यहाँ कभी नहीं दिखते।

उपयोग का उदाहरण:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

डिवाइस का Android संस्करण, मॉडल और चल रहा कर्नेल पढ़ते हैं।

उपयोग का उदाहरण:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo टेक्स्ट या किसी वेरिएबल का मान छापता है; env शेल के सभी वेरिएबल दिखाता है।

उपयोग का उदाहरण:

```
echo $HOME
env
```

---

16- Tar

फ़ाइलों और फ़ोल्डरों को एक संपीड़ित आर्काइव में पैक करता है, या किसी आर्काइव को खोलता है।

उपयोग का उदाहरण:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

किसी फ़ाइल का SHA-256 डाइजेस्ट छापता है, ताकि आप जाँच सकें कि यह ठीक वही फ़ाइल है जिसकी आपको उम्मीद है।

उपयोग का उदाहरण:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear स्क्रीन साफ़ करता है; exit शेल सत्र बंद करता है।

उपयोग का उदाहरण:

```
clear
exit
```
"#;

const INDONESIAN: &str = r#"@@ Penggunaan Terminal
@: Perintah Yang Dijalankan Terminal Bawaan Di Perangkat Ini
1- Pwd

Menampilkan folder tempat terminal bekerja. Sesi baru dibuka di proyek asal Anda, atau di folder utama aplikasi jika tidak ada proyek yang terbuka.

Contoh penggunaan:

```
pwd
```

---

2- Ls -La

Menampilkan isi folder saat ini, termasuk file tersembunyi, beserta izin, ukuran, dan waktu perubahan terakhir.

Contoh penggunaan:

```
ls -la
```

---

3- Cd

Pindah ke folder lain. cd .. naik satu tingkat dan cd ~ kembali ke folder utama.

Contoh penggunaan:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Membuat folder beserta folder induk yang belum ada.

Contoh penggunaan:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Menyalin file, atau dengan -r seluruh folder. Aslinya tetap di tempat.

Contoh penggunaan:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Memindahkan file atau folder ke tempat lain, atau mengganti namanya di tempat.

Contoh penggunaan:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Menghapus file, atau dengan -r seluruh folder. Tidak ada yang masuk ke tempat sampah, jadi baca namanya sekali lagi sebelum menekan Enter.

Contoh penggunaan:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Menampilkan isi file: cat mencetak semuanya, head baris-baris awal, dan tail baris-baris terakhir.

Contoh penggunaan:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Mencari teks di semua file dalam folder dan mencetak nama file serta nomor baris setiap kecocokan.

Contoh penggunaan:

```
grep -rn "onCreate" Java
```

---

10- Find

Mencari file berdasarkan nama di bawah folder saat ini. Nama boleh memakai * sebagai karakter pengganti.

Contoh penggunaan:

```
find . -name "*.xml"
```

---

11- Wc -L

Menghitung jumlah baris sebuah file.

Contoh penggunaan:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du menunjukkan ruang yang dipakai sebuah folder; df menunjukkan sisa ruang kosong di penyimpanan.

Contoh penggunaan:

```
du -sh .
df -h
```

---

13- Ps / Top

Menampilkan proses yang dijalankan aplikasi ini. Android hanya menunjukkan proses milik aplikasi itu sendiri, jadi aplikasi lain tidak pernah muncul di sini.

Contoh penggunaan:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Membaca versi Android dan model perangkat, serta kernel yang sedang berjalan.

Contoh penggunaan:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo mencetak teks atau nilai variabel; env menampilkan semua variabel yang dimiliki shell.

Contoh penggunaan:

```
echo $HOME
env
```

---

16- Tar

Mengemas file dan folder menjadi satu arsip terkompresi, atau membongkar sebuah arsip.

Contoh penggunaan:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Mencetak digest SHA-256 sebuah file, untuk memastikan itu memang file yang Anda harapkan.

Contoh penggunaan:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear membersihkan layar; exit mengakhiri sesi shell.

Contoh penggunaan:

```
clear
exit
```
"#;

const BRAZILIAN: &str = r#"@@ Uso Do Terminal
@: Comandos Que O Terminal Integrado Executa Neste Aparelho
1- Pwd

Mostra a pasta em que o terminal está trabalhando. Uma nova sessão abre no projeto de onde você veio, ou na pasta inicial do app quando nenhum projeto está aberto.

Exemplo de uso:

```
pwd
```

---

2- Ls -La

Lista o conteúdo da pasta atual, incluindo arquivos ocultos, com permissões, tamanho e a hora da última alteração.

Exemplo de uso:

```
ls -la
```

---

3- Cd

Entra em outra pasta. cd .. sobe um nível e cd ~ volta para a pasta inicial.

Exemplo de uso:

```
cd Java
cd ..
cd ~
```

---

4- Mkdir -P

Cria uma pasta junto com as pastas superiores que ainda não existem.

Exemplo de uso:

```
mkdir -p Java/com/example/app
```

---

5- Cp -R

Copia um arquivo, ou com -r uma pasta inteira. O original continua onde está.

Exemplo de uso:

```
cp Builder.json Builder.json.bak
cp -r res res_copy
```

---

6- Mv

Move um arquivo ou uma pasta para outro lugar, ou renomeia no mesmo lugar.

Exemplo de uso:

```
mv Old.java New.java
mv notes.txt Docs/
```

---

7- Rm -R

Apaga um arquivo, ou com -r uma pasta inteira. Nada vai para a lixeira, então leia o nome mais uma vez antes de apertar Enter.

Exemplo de uso:

```
rm Old.java
rm -r res_copy
```

---

8- Cat / Head / Tail

Mostram o conteúdo de um arquivo: cat imprime tudo, head as primeiras linhas e tail as últimas.

Exemplo de uso:

```
cat AndroidManifest.xml
head -n 20 Builder.json
tail -n 20 Builder.json
```

---

9- Grep -Rn

Procura um texto em todos os arquivos de uma pasta e imprime o nome do arquivo e o número da linha de cada ocorrência.

Exemplo de uso:

```
grep -rn "onCreate" Java
```

---

10- Find

Encontra arquivos pelo nome dentro da pasta atual. O nome pode ter * como curinga.

Exemplo de uso:

```
find . -name "*.xml"
```

---

11- Wc -L

Conta as linhas de um arquivo.

Exemplo de uso:

```
wc -l AndroidManifest.xml
```

---

12- Du / Df

du mostra quanto espaço uma pasta ocupa; df mostra quanto espaço livre resta no armazenamento.

Exemplo de uso:

```
du -sh .
df -h
```

---

13- Ps / Top

Listam os processos que este app iniciou. O Android mostra a um app apenas os próprios processos, então outros apps nunca aparecem aqui.

Exemplo de uso:

```
ps -A
top -n 1
```

---

14- Getprop / Uname

Leem a versão do Android e o modelo do aparelho, e o kernel em execução.

Exemplo de uso:

```
getprop ro.build.version.release
getprop ro.product.model
uname -a
```

---

15- Echo / Env

echo imprime um texto ou o valor de uma variável; env lista todas as variáveis do shell.

Exemplo de uso:

```
echo $HOME
env
```

---

16- Tar

Junta arquivos e pastas em um único arquivo compactado, ou extrai um.

Exemplo de uso:

```
tar -czf project.tar.gz Java res
tar -xzf project.tar.gz
```

---

17- Sha256sum

Imprime o resumo SHA-256 de um arquivo, para confirmar que é exatamente o arquivo esperado.

Exemplo de uso:

```
sha256sum AndroidManifest.xml
```

---

18- Clear / Exit

clear limpa a tela; exit encerra a sessão do shell.

Exemplo de uso:

```
clear
exit
```
"#;
