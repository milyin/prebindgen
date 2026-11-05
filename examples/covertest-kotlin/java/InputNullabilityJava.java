import io.prebindgen.covertest.JniErrorHandler;
import io.prebindgen.covertest.nullability.FunctionsKt;
import io.prebindgen.covertest.nullability.NullabilityHandle;
import java.math.BigInteger;
import java.util.Collections;

public class InputNullabilityJava {
    private static void rejectsNull(Runnable body) {
        try { body.run(); }
        catch (NullPointerException expected) { return; }
        throw new AssertionError("required null did not fail at the Kotlin boundary");
    }
    public static void main(String[] args) {
        JniErrorHandler<Long> fail = message -> { throw new AssertionError("unexpected binding error: " + message); };
        JniErrorHandler<BigInteger> wideFail = message -> { throw new AssertionError(message); };
        JniErrorHandler<String> textFail = message -> { throw new AssertionError(message); };
        rejectsNull(() -> FunctionsKt.nullabilityHandleRead(null, fail));
        rejectsNull(() -> FunctionsKt.nullabilityText(null, fail));
        rejectsNull(() -> FunctionsKt.nullabilityBytes(null, fail));
        rejectsNull(() -> FunctionsKt.nullabilityWide(null, wideFail));
        rejectsNull(() -> FunctionsKt.nullabilityTextCallback(null, textFail));
        rejectsNull(() -> FunctionsKt.nullabilityValues(null, fail));
        rejectsNull(() -> FunctionsKt.nullabilityValues(Collections.singletonList(null), fail));
        if (FunctionsKt.nullabilityHandleOptional(null, fail) != -1L) throw new AssertionError();
        if (FunctionsKt.nullabilityTextOptional(null, fail) != -1L) throw new AssertionError();
        if (FunctionsKt.nullabilityBytesOptional(null, fail) != -1L) throw new AssertionError();
        if (FunctionsKt.nullabilityWideOptional(null, wideFail) != null) throw new AssertionError();
        if (FunctionsKt.nullabilityValuesOptional(null, fail) != -1L) throw new AssertionError();
        if (FunctionsKt.nullabilityHandlesOptional(null, fail) != -1L) throw new AssertionError();
        NullabilityHandle handle = FunctionsKt.nullabilityHandleNew(13L, message -> { throw new AssertionError(message); });
        handle.close();
        final boolean[] closedFailed = {false};
        if (FunctionsKt.nullabilityHandleRead(handle, message -> {closedFailed[0] = true; return -7L;}) != -7L || !closedFailed[0])
            throw new AssertionError("closed handles must retain binding-error routing");
        System.out.println("Java nullability PASS: required null throws; Option null and runtime error callbacks are preserved");
    }
}
